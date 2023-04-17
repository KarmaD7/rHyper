use core::marker::PhantomData;

use virtio_drivers::{transport::{Transport, mmio::VirtIOHeader}, Hal, device::blk::VirtIOBlk as InnerDev};

struct VirtIOBlkBackend<'a, H: Hal, T: Transport> {
    header: &'a VirtIOHeader,
    inner: InnerDev<H, T>,
}

impl<'a, H: Hal, T: Transport> VirtIOBlkBackend<'a, H, T> {
    
}

pub fn init() {
    
}