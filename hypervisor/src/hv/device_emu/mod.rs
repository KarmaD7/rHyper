use alloc::{sync::Arc, vec, vec::Vec};

mod pl011;

pub trait MMIODevice: Send + Sync {
    fn mem_range(&self) -> core::ops::Range<usize>;
    fn read(&self, addr: usize, access_size: u8) -> rvm::RvmResult<u32>;
    fn write(&self, addr: usize, val: u32, access_size: u8) -> rvm::RvmResult;
}

pub struct VirtDeviceList {
    mmio_devices: Vec<Arc<dyn MMIODevice>>,
}

lazy_static::lazy_static! {
    static ref VIRT_DEVICES: VirtDeviceList = VirtDeviceList {
        mmio_devices: vec![
            Arc::new(pl011::Pl011::new(0x0900_0000)),
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
