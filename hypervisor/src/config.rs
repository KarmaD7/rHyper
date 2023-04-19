use spin::Mutex;

pub const BOOT_KERNEL_STACK_SIZE: usize = 0x4000;

pub const KERNEL_HEAP_SIZE: usize = 0x40_0000;

pub const PHYS_VIRT_OFFSET: usize = 0x40_000_000;
pub const PHYS_MEMORY_END: usize = 0x60_000_000;

pub const CPU_NUM: usize = 1;
pub const GUEST_NUM: usize = 1;

pub const PRIMARY_CPU_ID: usize = 0;

pub const BLK_QUEUE_SIZE: usize = 16;