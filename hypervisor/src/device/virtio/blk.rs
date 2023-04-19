use core::{marker::PhantomData, ptr::NonNull};

use virtio_drivers::{transport::{Transport, mmio::VirtIOHeader}, Hal, device::blk::VirtIOBlk as InnerDev};

use super::VirtIODevice;

struct VirtIOBlkDevice<H: Hal, T: Transport> {
    // header: Nonnu VirtIOHeader,
    header: NonNull<VirtIOHeader>,
    transport: T,
    inner: InnerDev<H, T>,
}

impl<H: Hal, T: Transport> VirtIODevice for VirtIOBlkDevice<H, T> {
    fn notify_handler() -> () {
        todo!()
    }
}

impl<H: Hal, T: Transport> VirtIOBlkDevice<H, T> {
    fn new() -> Self {
        todo!()
    }
}

pub fn init() {
    
}