use alloc::{sync::Arc, vec, vec::Vec};

use crate::{
    config::{CPU_NUM, GUEST_NUM},
    hv::gconfig::{VIRTIO_HEADER_EACH_SIZE, VIRTIO_HEADER_TOTAL_SIZE},
};

use super::gpm::GuestPhysMemorySet;

mod pl011;
mod vgic;
mod virtio;
mod dummy;
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
    // TODO: attach different devices to VM

    static ref VIRT_DEVICES: [VirtDeviceList; GUEST_NUM] = [VirtDeviceList {
        mmio_devices: vec![
            Arc::new(pl011::Pl011::new(0x0900_0000)),
            Arc::new(vgic::Vgic::new(0x0800_0000)),
            Arc::new(dummy::Dummy::new(0x0a00_0000, 0x3e00)),
            Arc::new(virtio::Virtio::new(0x0a00_3e00)),
        ]},
            // let virtio_nums = VIRTIO_HEADER_TOTAL_SIZE / VIRTIO_HEADER_EACH_SIZE;
            // let mut virtio_base = 0x0a00_0000;
            // for i in 0..virtio_nums {
            //     device_regions.push(Arc::new(virtio::Virtio::new(virtio_base)));
            //     virtio_base += VIRTIO_HEADER_EACH_SIZE;
            // }
            // device_regions,
        VirtDeviceList {
            mmio_devices: vec![
                Arc::new(pl011::Pl011::new(0x0900_0000)),
                Arc::new(vgic::Vgic::new(0x0800_0000)),
                Arc::new(dummy::Dummy::new(0x0a00_0000, 0x3c00)),
                Arc::new(virtio::Virtio::new(0x0a00_3c00)),
                Arc::new(dummy::Dummy::new(0x0a00_3e00, 0x200)),
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
