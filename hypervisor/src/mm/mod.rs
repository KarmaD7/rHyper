pub mod address;

pub mod frame;
mod heap;

pub use address::{PhysAddr, VirtAddr};

pub const PAGE_SIZE: usize = 0x1000;

pub fn init_heap_early() {
    heap::init();
}

pub fn init() {
    frame::init();
}
