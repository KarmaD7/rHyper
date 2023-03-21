pub const BOOT_KERNEL_STACK_SIZE: usize = 0x4000;

pub const KERNEL_HEAP_SIZE: usize = 0x40_0000;

pub const PHYS_VIRT_OFFSET: usize = 0x40_000_000;
pub const PHYS_MEMORY_END: usize = 0x48_000_000;

pub const CPU_NUM: usize = 4;