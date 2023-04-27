use alloc::{sync::Arc, vec, vec::Vec};

use super::gpm::GuestPhysMemorySet;

mod pl011;
mod vgic;
mod virtio;
// mod virt_queue;

pub trait MMIODevice: Send + Sync {
    fn mem_range(&self) -> core::ops::Range<usize>;
    fn read(&self, addr: usize, access_size: u8) -> rvm::RvmResult<u32>;
    fn write(
        &self,
        addr: usize,
        val: u32,
        access_size: u8,
        gpm: &GuestPhysMemorySet,
    ) -> rvm::RvmResult;
}

pub struct VirtDeviceList {
    mmio_devices: Vec<Arc<dyn MMIODevice>>,
}

lazy_static::lazy_static! {
    static ref VIRT_DEVICES: VirtDeviceList = VirtDeviceList {
        mmio_devices: vec![
            Arc::new(pl011::Pl011::new(0x0900_0000)),
            // todo: use marco
            Arc::new(virtio::Virtio::new(0x0a00_0000)),
            Arc::new(virtio::Virtio::new(0x0a00_0200)),
            Arc::new(virtio::Virtio::new(0x0a00_0400)),
            Arc::new(virtio::Virtio::new(0x0a00_0600)),
            Arc::new(virtio::Virtio::new(0x0a00_0800)),
            Arc::new(virtio::Virtio::new(0x0a00_0a00)),
            Arc::new(virtio::Virtio::new(0x0a00_0c00)),
            Arc::new(virtio::Virtio::new(0x0a00_0e00)),
            Arc::new(virtio::Virtio::new(0x0a00_1000)),
            Arc::new(virtio::Virtio::new(0x0a00_1200)),
            Arc::new(virtio::Virtio::new(0x0a00_1400)),
            Arc::new(virtio::Virtio::new(0x0a00_1600)),
            Arc::new(virtio::Virtio::new(0x0a00_1800)),
            Arc::new(virtio::Virtio::new(0x0a00_1a00)),
            Arc::new(virtio::Virtio::new(0x0a00_1c00)),
            Arc::new(virtio::Virtio::new(0x0a00_1e00)),
            Arc::new(virtio::Virtio::new(0x0a00_2000)),
            Arc::new(virtio::Virtio::new(0x0a00_2200)),
            Arc::new(virtio::Virtio::new(0x0a00_2400)),
            Arc::new(virtio::Virtio::new(0x0a00_2600)),
            Arc::new(virtio::Virtio::new(0x0a00_2800)),
            Arc::new(virtio::Virtio::new(0x0a00_2a00)),
            Arc::new(virtio::Virtio::new(0x0a00_2c00)),
            Arc::new(virtio::Virtio::new(0x0a00_2e00)),
            Arc::new(virtio::Virtio::new(0x0a00_3000)),
            Arc::new(virtio::Virtio::new(0x0a00_3200)),
            Arc::new(virtio::Virtio::new(0x0a00_3400)),
            Arc::new(virtio::Virtio::new(0x0a00_3600)),
            Arc::new(virtio::Virtio::new(0x0a00_3800)),
            Arc::new(virtio::Virtio::new(0x0a00_3a00)),
            Arc::new(virtio::Virtio::new(0x0a00_3c00)),
            Arc::new(virtio::Virtio::new(0x0a00_3e00)),
        ]
    };
}

impl VirtDeviceList {
    pub fn find_mmio_device(&self, addr: usize) -> Option<&Arc<dyn MMIODevice>> {
        self.mmio_devices
            .iter()
            .find(|dev| dev.mem_range().contains(&addr))
    }
}

pub fn all_virt_devices() -> &'static VirtDeviceList {
    &VIRT_DEVICES
}
