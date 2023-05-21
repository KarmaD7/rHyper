pub const BOOT_KERNEL_STACK_SIZE: usize = 0x4000;

pub const KERNEL_HEAP_SIZE: usize = 0x40_0000;

pub const PHYS_VIRT_OFFSET: usize = 0x4000_0000;
pub const PHYS_MEMORY_END: usize = 0x6000_0000;

pub const CPU_NUM: usize = 1;
pub const VM_NUM: usize = 1;
pub const CPU_TO_VM: [usize; CPU_NUM] = [0];

pub const PRIMARY_CPU_ID: usize = 0;

pub const BLK_QUEUE_SIZE: usize = 16;

pub static mut GUEST_ENTRIES: [usize; CPU_NUM] = [0; CPU_NUM];
pub static mut PSCI_CONTEXT: [usize; CPU_NUM] = [0; CPU_NUM];
