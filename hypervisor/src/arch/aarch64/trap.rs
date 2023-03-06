use core::arch::global_asm;

use aarch64_cpu::registers::*;
use tock_registers::interfaces::Writeable;

global_asm!(include_str!("trap.S"));

pub fn init() {
    extern "C" {
        fn exception_vector_base();
    }
    VBAR_EL2.set(exception_vector_base as usize as _);
}
