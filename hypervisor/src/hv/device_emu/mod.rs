use alloc::{sync::Arc, vec, vec::Vec};

use crate::{
    config::VM_NUM,
};

use super::gpm::GuestPhysMemorySet;

mod dummy;
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
    static ref VIRT_DEVICES: [VirtDeviceList; VM_NUM] = [
        VirtDeviceList {
            mmio_devices: vec![
                Arc::new(pl011::Pl011::new(0x0900_0000)),
                Arc::new(vgic::Vgic::new(0x0800_0000)),
                Arc::new(dummy::Dummy::new(0x0a00_0000, 0x3e00)),
                Arc::new(virtio::Virtio::new(0x0a00_3e00)),
        ]},

        VirtDeviceList {
            mmio_devices: vec![
                Arc::new(pl011::Pl011::new(0x0900_0000)),
                Arc::new(vgic::Vgic::new(0x0800_0000)),
                Arc::new(dummy::Dummy::new(0x0a00_0000, 0x3c00)),
                Arc::new(virtio::Virtio::new(0x0a00_3c00)),
                Arc::new(dummy::Dummy::new(0x0a00_3e00, 0x200)),
            ]
        },

        VirtDeviceList {
            mmio_devices: vec![
                Arc::new(pl011::Pl011::new(0x0900_0000)),
                Arc::new(vgic::Vgic::new(0x0800_0000)),
            ]
        },
     ];
}

impl VirtDeviceList {
    pub fn find_mmio_device(&self, addr: usize) -> Option<&Arc<dyn MMIODevice>> {
        self.mmio_devices
            .iter()
            .find(|dev| dev.mem_range().contains(&addr))
    }
}

pub fn all_virt_devices(vm_id: usize) -> &'static VirtDeviceList {
    &VIRT_DEVICES[vm_id]
}
