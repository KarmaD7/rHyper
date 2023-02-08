#[macro_use]
pub mod regs;

mod virt;

pub use virt::{ArchPerCpuState, RvmVcpu};

pub fn has_hardware_support() -> bool {
  true
}