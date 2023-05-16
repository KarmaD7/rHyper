pub const BOOT_KERNEL_STACK_SIZE: usize = 0x4000;

pub const KERNEL_HEAP_SIZE: usize = 0x40_0000;

pub const PHYS_VIRT_OFFSET: usize = 0x4000_0000;
pub const PHYS_MEMORY_END: usize = 0x8000_0000;

pub const CPU_NUM: usize = 4;
pub const VM_NUM: usize = 3;
pub const CPU_TO_VM: [usize; CPU_NUM] = [0, 1, 2, 2];

pub const PRIMARY_CPU_ID: usize = 0;

pub const BLK_QUEUE_SIZE: usize = 16;

pub static mut GUEST_ENTRIES: [usize; CPU_NUM] = [0; CPU_NUM];
pub static mut PSCI_CONTEXT: [usize; CPU_NUM] = [0; CPU_NUM];
