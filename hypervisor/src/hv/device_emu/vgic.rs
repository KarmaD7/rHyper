use rvm::RvmResult;
use tock_registers::{register_structs, registers::{ReadWrite, ReadOnly, WriteOnly}};

use super::MMIODevice;

pub struct Vgic {
    base_vaddr: usize,
    gicd_size: usize,
    // TODO
    enabled: bool,

}

// copy from gicv2.rs.
// need to be removed.
register_structs! {
    #[allow(non_snake_case)]
    GicDistributorRegs {
        /// Distributor Control Register.
        (0x0000 => CTLR: ReadWrite<u32>),
        /// Interrupt Controller Type Register.
        (0x0004 => TYPER: ReadOnly<u32>),
        /// Distributor Implementer Identification Register.
        (0x0008 => IIDR: ReadOnly<u32>),
        (0x000c => _reserved_0),
        /// Interrupt Group Registers.
        (0x0080 => IGROUPR: [ReadWrite<u32>; 0x20]),
        /// Interrupt Set-Enable Registers.
        (0x0100 => ISENABLER: [ReadWrite<u32>; 0x20]),
        /// Interrupt Clear-Enable Registers.
        (0x0180 => ICENABLER: [ReadWrite<u32>; 0x20]),
        /// Interrupt Set-Pending Registers.
        (0x0200 => ISPENDR: [ReadWrite<u32>; 0x20]),
        /// Interrupt Clear-Pending Registers.
        (0x0280 => ICPENDR: [ReadWrite<u32>; 0x20]),
        /// Interrupt Set-Active Registers.
        (0x0300 => ISACTIVER: [ReadWrite<u32>; 0x20]),
        /// Interrupt Clear-Active Registers.
        (0x0380 => ICACTIVER: [ReadWrite<u32>; 0x20]),
        /// Interrupt Priority Registers.
        (0x0400 => IPRIORITYR: [ReadWrite<u32>; 0x100]),
        /// Interrupt Processor Targets Registers.
        (0x0800 => ITARGETSR: [ReadWrite<u32>; 0x100]),
        /// Interrupt Configuration Registers.
        (0x0c00 => ICFGR: [ReadWrite<u32>; 0x40]),
        (0x0d00 => _reserved_1),
        /// Software Generated Interrupt Register.
        (0x0f00 => SGIR: WriteOnly<u32>),
        (0x0f04 => @END),
    }
}

impl Vgic {
    pub const fn new(base_vaddr: usize, gicd_size: usize) -> Self {
        Self { 
            base_vaddr, 
            gicd_size,
            enabled: false,
        }
    }
}

impl MMIODevice for Vgic {
    fn mem_range(&self) -> core::ops::Range<usize> {
        self.base_vaddr..self.base_vaddr + self.gicd_size
    }

    fn read(&self, addr: usize, access_size: u8) -> RvmResult<u32> {
        // todo!()
        match addr - self.base_vaddr {
            CTLR => {
                Ok(self.enabled as u32)
            },
            TYPER => {
                Ok(1)
            },
            IIDR => {
                Ok(0)
            },
            _ => {
                Err(rvm::RvmError::InvalidParam)
            },
        }
    }

    fn write(&self, addr: usize, val: u32, access_size: u8) -> RvmResult {
        todo!()
    }
}