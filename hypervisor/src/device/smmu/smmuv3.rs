use rvm::RvmResult;
use spin::Mutex;
// maybe TODO
use tock_registers::{register_structs, registers::{ReadWrite, ReadOnly}, interfaces::Readable};

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
        /// SID Size.
        const SIDSIZE = 0b111111 << 0;
        /// Event Queue Size.
        const EVTQS = 0b11111 << 16;
        /// Command Queue Size.
        const CMDQS = 0b11111 << 21;
        /// Rel.
        const REL = 1 << 28;
        /// Queues Are Preseted.
        const QUEUES_PRESET = 1 << 29;
        /// Tables Are Preseted.
        const TABLES_PRESET = 1 << 30;
    }
}

impl IDR1Features {
    const SIDSIZE_MASK: u32 = 0b111111;
    const EVTQS_MASK: u32 = 0b11111;
    const CMDQS_MASK: u32 = 0x11111;
}

struct Smmu {
    base_vaddr: VirtAddr,
    // store some shadow registers in memory
    // strtab_base: u64,
    // strtab_base_config: u64,
    features: IDR0Features,
    cmd_queue_size: u32,
    event_queue_size: u32,
    // cmd_queue: Mutex<SmmuQueue>,
    // event_queue: SmmuQueue, // queue size will be known at runtime

    // ste_tables: todo 
}

impl Smmu {
    pub fn new(base_vaddr: VirtAddr) -> Self {
        Self { base_vaddr, features: IDR0Features::empty(), cmd_queue_size: 0, event_queue_size: 0 }
    }

    const fn regs(&self) -> &Smmuv3Regs {
        unsafe { &*(self.base_vaddr as *const _)}
    }

    fn init_features(&mut self) -> RvmResult {
        self.features = IDR0Features::from_bits_truncate(self.regs().IDR0.get());
        if !self.features.contains(IDR0Features::S2P) {
            return Err(rvm::RvmError::Unsupported);
        }

        let idr1_features = IDR1Features::from_bits_truncate(self.regs().IDR1.get());
        if idr1_features.intersects(IDR1Features::QUEUES_PRESET | IDR1Features::TABLES_PRESET | IDR1Features::REL) {
            return Err(rvm::RvmError::Unsupported);
        }

        self.cmd_queue_size = (idr1_features.bits() & IDR1Features::CMDQS.bits()) >> 21;
        self.event_queue_size = (idr1_features.bits() & IDR1Features::EVTQS.bits()) >> 16;

        Ok(())
    }

    fn setup_stream_table(&mut self) -> RvmResult {
        
        Ok(())
        // only one-level stream table is supported now.

    }

    fn setup_queue(&mut self) -> RvmResult {
        
        Ok(())
    }

    fn setup_smmu_device(&self) -> RvmResult {
        Ok(())
    }
 
    fn init(&mut self) -> RvmResult {
        self.init_features()?;
        self.setup_queue()?;
        self.setup_stream_table()?;
        self.setup_smmu_device()?;
        Ok(())
    }
}

pub fn init() {

}