use rvm::RvmResult;
use spin::Mutex;
use tock_registers::{
    register_structs,
    registers::{ReadOnly, ReadWrite, WriteOnly},
};

use crate::hv::gpm::GuestPhysMemorySet;

use super::MMIODevice;

pub struct Dummy {
    base_vaddr: usize,
    dummy_size: usize,
    // TODO
}

impl Dummy {
    pub const fn new(base_vaddr: usize, dummy_size: usize) -> Self {
        Self {
            base_vaddr,
            dummy_size,
        }
    }
}

impl MMIODevice for Dummy {
    fn mem_range(&self) -> core::ops::Range<usize> {
        self.base_vaddr..self.base_vaddr + self.dummy_size
    }

    fn read(&self, addr: usize, access_size: u8) -> RvmResult<u32> {
        Ok(0)
    }

    fn write(&self, addr: usize, val: u32, access_size: u8, _: &GuestPhysMemorySet) -> RvmResult {
        Ok(())
    }
}
