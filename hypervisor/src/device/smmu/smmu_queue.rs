use core::ptr::NonNull;

use crate::mm::VirtAddr;

struct QueueEntry {
    idx: usize,
}

struct EventQueueEntry {
    idx: usize,
}

// trait

pub struct SmmuQueue {
    smmu_base_addr: VirtAddr,
    queue: Option<NonNull<QueueEntry>>,
    head: u32,
    tail: u32,
    // max_n_shift: u32,
}

impl SmmuQueue {
    // fn new(base_vaddr) {

    // }
    fn push() {}
}
