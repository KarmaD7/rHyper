use rvm::{GuestPhysAddr, HostPhysAddr};
use spin::Mutex;

use crate::config::CPU_NUM;

use super::gpm::GuestPhysMemorySet;

pub const BIOS_PADDR: HostPhysAddr = 0x400_0000;
pub const BIOS_SIZE: usize = 0x1000;

pub const GUEST_IMAGE_PADDR: HostPhysAddr = 0x400_1000;
pub const GUEST_IMAGE_SIZE: usize = 0x10_0000; // 1M

pub const GUEST_PHYS_MEMORY_BASE: GuestPhysAddr = 0x4000_0000;
pub const BIOS_ENTRY: GuestPhysAddr = 0x8000;
pub const GUEST_ENTRY: GuestPhysAddr = 0x4008_0000;
pub const GUEST_PHYS_MEMORY_SIZE: usize = 0x800_0000; // 128M

pub const VIRTIO_HEADER_TOTAL_SIZE: usize = 0x4000;
pub const VIRTIO_HEADER_EACH_SIZE: usize = 0x200;

pub const VIRTIO_BLK_IDX: usize = 0;
pub const VIRTIO_NET_IDX: usize = 1;

const NONE: Option<GuestPhysMemorySet> = None;
pub static GUEST_GPM: Mutex<[Option<GuestPhysMemorySet>; CPU_NUM]> = Mutex::new([NONE; CPU_NUM]);
