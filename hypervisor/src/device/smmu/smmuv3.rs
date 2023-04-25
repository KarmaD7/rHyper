use rvm::RvmResult;
// maybe TODO
use tock_registers::{register_structs, registers::{ReadWrite, ReadOnly}};

use crate::mm::VirtAddr;

use super::smmu_queue::SmmuQueue;



register_structs! {
    #[allow(non_snake_case)]
    Smmuv3Regs {
        (0x0000 => IDR0: ReadOnly<u32>),
        (0x0004 => IDR1: ReadOnly<u32>),
        (0x0008 => IDR2: ReadOnly<u32>),

        (0x000c => @END),
    }
}

bitflags::bitflags! {
    // In SMMU_IDR0.
    // Only these features are used in jailhouse.
    struct IDR0Features: u32 {
        /// Stage 2 translation supported.
        const S2P = 1 << 0;
        /// Stage 1 translation supported.
        const S1P = 1 << 1;
        /// 16-bit VMID supported.
        const VMID16 = 1 << 18;
        /// Two Level Stream Table Supported.
        const ST_LVL = 1 << 27;
    }
}

bitflags::bitflags! {
    // In SMMU_IDR1.
    // Only these features are used in jailhouse.
    struct IDR1Features: u32 {
        /// Stage 2 translation supported.
        const S2P = 1 << 0;
        /// Stage 1 translation supported.
        const S1P = 1 << 1;
        /// 16-bit VMID supported.
        const VMID16 = 1 << 18;
        /// Two Level Stream Table Supported.
        const ST_LVL = 1 << 27;
    }
}

struct Smmu {
    base_vaddr: VirtAddr,
    features: IDR0Features,
    // cmd_queue: SmmuQueue,
    // event_queue: SmmuQueue,
    // ste_tables: todo 
}

impl Smmu {
    const fn new(base_vaddr: VirtAddr) -> Self {
        Self { base_vaddr, features: IDR0Features::empty() }
    }

    const fn regs(&self) -> &Smmuv3Regs {
        unsafe { &*(self.base_vaddr as *const _)}
    }

    fn init_features(&mut self) -> RvmResult {
        Ok(())
        
    }

    fn setup_stream_table(&self) -> RvmResult {
        Ok(())
    }

    fn setup_queue(&self) -> RvmResult {
        Ok(())
    }

    fn setup_smmu_device(&self) -> RvmResult {
        Ok(())
    }
 
    fn init(&self) -> RvmResult {
        self.init_features()?;
        self.setup_queue()?;
        self.setup_stream_table()?;
        self.setup_smmu_device()?;
        Ok(())
    }
}

pub fn init() {

}