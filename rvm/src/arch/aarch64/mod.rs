#[macro_use]
pub mod regs;

mod vcpu;

use core::marker::PhantomData;

use crate::RvmHal;
pub use vcpu::ArmVcpu as RvmVcpu;
pub use self::ArmPerCpuState as ArchPerCpuState;

pub fn has_hardware_support() -> bool {
  true
}

pub struct ArmPerCpuState<H: RvmHal> {
  _phantom_data: PhantomData<H>
}