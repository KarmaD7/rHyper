pub mod mp;
pub mod psci;

use aarch64_cpu::{asm, asm::barrier, registers::*};
use rvm::{GenericPTE, MemFlags, Stage1PTE};
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

use crate::arch::instructions;
use crate::config::{BOOT_KERNEL_STACK_SIZE, CPU_NUM};

#[link_section = ".bss.stack"]
static mut BOOT_STACK: [u8; BOOT_KERNEL_STACK_SIZE * CPU_NUM] =
    [0; BOOT_KERNEL_STACK_SIZE * CPU_NUM];

#[link_section = ".data.boot_page_table"]
static mut BOOT_PT_L0: [Stage1PTE; 512] = [Stage1PTE::empty(); 512];

#[link_section = ".data.boot_page_table"]
static mut BOOT_PT_L1: [Stage1PTE; 512] = [Stage1PTE::empty(); 512];

// static mut DTB_ADDR: usize = 0;

unsafe fn switch_to_el2() {
    SPSel.write(SPSel::SP::ELx);
    let current_el = CurrentEL.read(CurrentEL::EL);
    if current_el == 3 {
        SCR_EL3.write(
            SCR_EL3::NS::NonSecure + SCR_EL3::HCE::HvcEnabled + SCR_EL3::RW::NextELIsAarch64,
        );
        SPSR_EL3.write(
            SPSR_EL3::M::EL2h
                + SPSR_EL3::D::Masked
                + SPSR_EL3::A::Masked
                + SPSR_EL3::I::Masked
                + SPSR_EL3::F::Masked,
        );
        ELR_EL3.set(LR.get());
        SP_EL1.set(BOOT_STACK.as_ptr_range().end as u64);
        // This should be SP_EL2. To
        asm::eret();
    }
}

unsafe fn init_mmu() {
    // Device-nGnRE memory
    let attr0 = MAIR_EL2::Attr0_Device::nonGathering_nonReordering_EarlyWriteAck;
    // Normal memory
    let attr1 = MAIR_EL2::Attr1_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc
        + MAIR_EL2::Attr1_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc;
    MAIR_EL2.write(attr0 + attr1); // 0xff_04

    // Enable TTBR0 walks, page size = 4K, vaddr size = 48 bits, paddr size = 40 bits.
    let tcr_flags = TCR_EL2::TG0::KiB_4
        + TCR_EL2::SH0::Inner
        + TCR_EL2::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL2::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL2::T0SZ.val(16);
    TCR_EL2.write(TCR_EL2::PS::Bits_40 + tcr_flags);
    barrier::isb(barrier::SY);

    // Set both TTBR0 and TTBR1
    let root_paddr = BOOT_PT_L0.as_ptr() as usize as _;
    TTBR0_EL2.set(root_paddr);

    // Flush TLB
    instructions::flush_tlb_all();

    // Enable the MMU and turn on I-cache and D-cache
    SCTLR_EL2.modify(SCTLR_EL2::M::Enable + SCTLR_EL2::C::Cacheable + SCTLR_EL2::I::Cacheable);
    barrier::isb(barrier::SY);
}

unsafe fn init_boot_page_table() {
    // 0x0000_0000_0000 ~ 0x0080_0000_0000, table
    BOOT_PT_L0[0] = Stage1PTE::new_table(BOOT_PT_L1.as_ptr() as usize);
    // 0x0000_0000_0000..0x0000_4000_0000, 1G block, device memory
    BOOT_PT_L1[0] =
        Stage1PTE::new_page(0, MemFlags::READ | MemFlags::WRITE | MemFlags::DEVICE, true);
    // 0x0000_4000_0000..0x0000_8000_0000, 1G block, normal memory
    BOOT_PT_L1[1] = Stage1PTE::new_page(
        0x4000_0000,
        MemFlags::READ | MemFlags::WRITE | MemFlags::EXECUTE,
        true,
    );
}

#[naked]
#[no_mangle]
#[link_section = ".text.boot"]
unsafe extern "C" fn _start() -> ! {
    // PC = 0x4008_0000
    // x0 = DTB_ADDR
    core::arch::asm!("
        mov     x21, x0
        adrp    x8, boot_stack_top
        mov     sp, x8

        bl      {switch_to_el2}
        bl      {init_boot_page_table}
        bl      {init_mmu}

        ldr     x8, =boot_stack_top
        mov     sp, x8

        mrs     x0, mpidr_el1
        and     x0, x0, #0xffffff

        mov     x1, x21
        ldr     x8, ={rust_main}
        blr     x8
        b      .",

        switch_to_el2 = sym switch_to_el2,
        init_boot_page_table = sym init_boot_page_table,
        init_mmu = sym init_mmu,
        rust_main = sym crate::rust_main,
        options(noreturn),
    )
}

#[naked]
#[no_mangle]
#[link_section = ".text.boot"]
unsafe extern "C" fn _start_secondary() -> ! {
    core::arch::asm!("
        adrp    x8, boot_stack_top
        mov     x1, #0
        msr     vmpidr_el2, x1

        mrs     x0, mpidr_el1
        and     x0, x0, #0xffffff
        mov     x19, {boot_stack_size}
        mul     x19, x0, x19        
        sub     x8, x8, x19
        mov     sp, x8

        bl      {switch_to_el2}
        bl      {init_boot_page_table}
        bl      {init_mmu}

        ldr     x8, =boot_stack_top
        sub     x8, x8, x19
        mov     sp, x8

        mrs     x0, mpidr_el1
        and     x0, x0, #0xffffff
        ldr     x8, ={rust_main_secondary}
        blr     x8
        b      .",
        switch_to_el2 = sym switch_to_el2,
        init_boot_page_table = sym init_boot_page_table,
        init_mmu = sym init_mmu,
        rust_main_secondary = sym crate::rust_main_secondary,
        boot_stack_size = const BOOT_KERNEL_STACK_SIZE,
        options(noreturn),
    )
}
