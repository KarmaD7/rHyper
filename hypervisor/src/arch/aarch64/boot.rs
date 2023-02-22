use core::arch::global_asm;

use aarch64_cpu::{asm, asm::barrier, registers::*};
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

global_asm!(include_str!("trap.S"));

unsafe fn switch_to_el2() {
  extern "C" {
    fn exception_vector_base();
  }
  VBAR_EL2.set(exception_vector_base as usize as _);
  // Disable EL1 timer traps and the timer offset.
  CNTHCTL_EL2.modify(CNTHCTL_EL2::EL1PCEN::SET + CNTHCTL_EL2::EL1PCTEN::SET);
  CNTVOFF_EL2.set(0);
  // Set EL1 to 64bit.
  HCR_EL2.write(HCR_EL2::RW::EL1IsAarch64);

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

#[naked]
#[no_mangle]
#[link_section = ".text.boot"]
unsafe extern "C" fn _start() -> ! {
    // PC = 0x4008_0000
    core::arch::asm!("
        adrp    x8, boot_stack_top
        mov     sp, x8
        bl      {switch_to_el2}
        ldr     x8, =boot_stack_top
        mov     sp, x8
        ldr     x8, ={rust_main}
        blr     x8
        b      .",
        switch_to_el2 = sym switch_to_el2,
        // init_boot_page_table = sym init_boot_page_table,
        // init_mmu = sym init_mmu,
        rust_main = sym crate::rust_main,
        options(noreturn),
    )
}
