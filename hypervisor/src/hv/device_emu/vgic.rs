use rvm::RvmResult;
use spin::Mutex;
use tock_registers::{
    register_structs,
    registers::{ReadOnly, ReadWrite, WriteOnly},
};

use crate::hv::gpm::GuestPhysMemorySet;

use super::MMIODevice;

pub struct Vgic {
    base_vaddr: usize,
    gicd_size: usize,
    // TODO
    inner: Mutex<VgicdInner>,
}

#[derive(Default)]
pub struct VgicdInner {
    enabled: bool,
}

impl VgicdInner {
    pub const fn new() -> Self {
        Self { enabled: false }
    }
}

// TODO: merge these consts with regs in gicv2.rs.
const GICD_CTLR: usize = 0x00;
const GICD_TYPER: usize = 0x04;
const GICD_IIDR: usize = 0x08;

// copy from gicv2.rs.
impl Vgic {
    pub const fn new(base_vaddr: usize, gicd_size: usize) -> Self {
        Self {
            base_vaddr,
            gicd_size,
            inner: Mutex::new(VgicdInner::new()),
        }
    }
}

impl MMIODevice for Vgic {
    fn mem_range(&self) -> core::ops::Range<usize> {
        self.base_vaddr..self.base_vaddr + self.gicd_size
    }

    fn read(&self, addr: usize, access_size: u8) -> RvmResult<u32> {
        // TODO: read SGI-related registers
        trace!("GICD read addr 0x{:x}, access size {}", addr, access_size);
        let val = match addr - self.base_vaddr {
            _ => unsafe { *(addr as *const u32) },
        };
        Ok(val)
    }

    fn write(&self, addr: usize, val: u32, access_size: u8, _: &GuestPhysMemorySet) -> RvmResult {
        trace!("GICD write addr 0x{:x}, access size {}", addr, access_size);
        // TODO: write SGI
        Ok(())
    }
}
