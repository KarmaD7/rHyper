#[macro_use]
pub mod regs;

mod ept;
mod instructions;
mod s1pt;
mod vcpu;

use core::marker::PhantomData;

pub use self::ept::ExtendedPageTable as NestedPageTable;
pub use self::s1pt::{PageTable, Stage1PTE};
pub use self::ArmPerCpuState as ArchPerCpuState;
use crate::{RvmHal, RvmResult};
use aarch64_cpu::registers::HCR_EL2;
use tock_registers::interfaces::{Writeable, ReadWriteable, Readable};
pub use vcpu::ArmVcpu as RvmVcpu;
pub use vcpu::{ArmExitInfo, ArmExitReason};

pub fn has_hardware_support() -> bool {
    true
}
pub struct ArmPerCpuState<H: RvmHal> {
    _phantom_data: PhantomData<H>,
}

impl<H: RvmHal> ArmPerCpuState<H> {
    pub const fn new() -> Self {
        Self {
            _phantom_data: PhantomData,
        }
    }

    pub fn is_enabled(&self) -> bool {
        info!("reading is enabled: {}", HCR_EL2.read(HCR_EL2::VM));
        HCR_EL2.read(HCR_EL2::VM) != 0
    }

    pub fn hardware_enable(&mut self) -> RvmResult {
        HCR_EL2.write(
            HCR_EL2::VM::Enable + HCR_EL2::RW::EL1IsAarch64 + HCR_EL2::AMO::SET + HCR_EL2::FMO::SET
        );
        Ok(())
    }

    pub fn hardware_disable(&mut self) -> RvmResult {
        HCR_EL2.modify(HCR_EL2::VM::Disable);
        Ok(())
    }
}
