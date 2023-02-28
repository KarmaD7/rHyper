use core::arch::global_asm;

use aarch64_cpu::{asm, asm::barrier, registers::*};
use rvm::{Stage1PTE, HostPhysAddr, GenericPTE, MemFlags};
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

use crate::config::BOOT_KERNEL_STACK_SIZE;

use super::instructions;

global_asm!(include_str!("trap.S"));

#[link_section = ".bss.stack"]
static mut BOOT_STACK: [u8; BOOT_KERNEL_STACK_SIZE] = [0; BOOT_KERNEL_STACK_SIZE];

#[link_section = ".data.boot_page_table"]
static mut BOOT_PT_L0: [Stage1PTE; 512] = [Stage1PTE::empty(); 512];

#[link_section = ".data.boot_page_table"]
static mut BOOT_PT_L1: [Stage1PTE; 512] = [Stage1PTE::empty(); 512];

unsafe fn switch_to_el2() {
  extern "C" {
    fn exception_vector_base();
  }
  VBAR_EL2.set(exception_vector_base as usize as _);
  // Disable EL1 timer traps and the timer offset.
  CNTHCTL_EL2.modify(CNTHCTL_EL2::EL1PCEN::SET + CNTHCTL_EL2::EL1PCTEN::SET);
  CNTVOFF_EL2.set(0);
  HCR_EL2.write(HCR_EL2::FWB::Enabled + HCR_EL2::VM::Enable + HCR_EL2::RW::EL1IsAarch64 + HCR_EL2::AMO::SET + HCR_EL2::FMO::SET);

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
    asm::eret();
  }
  // unimplemented!();
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
  BOOT_PT_L1[0] = Stage1PTE::new_page(
    0,
    MemFlags::READ | MemFlags::WRITE | MemFlags::DEVICE,
      true,
  );
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
    core::arch::asm!("
        adrp    x8, boot_stack_top
        mov     sp, x8
        bl      {switch_to_el2}
        bl      {init_boot_page_table}
        bl      {init_mmu}
        ldr     x8, =boot_stack_top
        mov     sp, x8
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
