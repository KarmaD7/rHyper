use core::arch::asm;

use aarch64_cpu::asm;
use spin::Mutex;

use crate::{hv::{gpm::GuestPhysMemorySet, gconfig::VIRTIO_HEADER_EACH_SIZE}, mm::PAGE_SIZE};

use super::MMIODevice;

const VIRTIO_LEGACY_PFN: usize = 0x40;
const VIRTIO_NOTIFY: usize = 0x50;
const VIRTIO_DESC_LOW: usize = 0x80;
const VIRTIO_DESC_HIGH: usize = 0x84;
const VIRTIO_DRIVER_LOW: usize = 0x90;
const VIRTIO_DRIVER_HIGH: usize = 0x94;
const VIRTIO_DEVICE_LOW: usize = 0xa0;
const VIRTIO_DEVICE_HIGH: usize = 0xa4;

pub struct Virtio {
    base_vaddr: usize,
    virt_queue_addr: Mutex<VirtQueueAddr>,
}

#[derive(Default)]
struct VirtQueueAddr {
    desc_gpa_low: Option<u32>,
    desc_gpa_high: Option<u32>,
    driver_gpa_low: Option<u32>,
    driver_gpa_high: Option<u32>,
    device_gpa_low: Option<u32>,
    device_gpa_high: Option<u32>,
    legacy_vqaddr: usize,
}

impl VirtQueueAddr {
    pub const fn new() -> Self {
        Self {
            desc_gpa_low: None,
            desc_gpa_high: None,
            driver_gpa_low: None,
            driver_gpa_high: None,
            device_gpa_low: None,
            device_gpa_high: None,
            legacy_vqaddr: 0,
        }
    }
}

impl Virtio {
    pub const fn new(base_vaddr: usize) -> Self {
        Self {
            base_vaddr,
            virt_queue_addr: Mutex::new(VirtQueueAddr::new()),
        }
    }

    fn write_vqaddr(&self, offset: usize, gpm: &GuestPhysMemorySet) {
        //todo: use macro
        let vqaddr = self.virt_queue_addr.lock();
        match offset {
            VIRTIO_DESC_LOW => {
                if let (Some(low), Some(high)) = (vqaddr.desc_gpa_low, vqaddr.desc_gpa_high) {
                    let gpaddr = ((high as usize) << 32) + low as usize;
                    info!("desc gpaddr is 0x{:x}", gpaddr);
                    let hpaddr = gpm.gpa_to_hpa(gpaddr);
                    unsafe {
                        *((self.base_vaddr + VIRTIO_DESC_LOW) as *mut u32) = hpaddr as u32;
                    }
                    unsafe {
                        *((self.base_vaddr + VIRTIO_DESC_HIGH) as *mut u32) = (hpaddr >> 32) as u32;
                    }
                }
            }
            VIRTIO_DEVICE_LOW => {
                if let (Some(low), Some(high)) = (vqaddr.device_gpa_low, vqaddr.device_gpa_high) {
                    let gpaddr = ((high as usize) << 32) + low as usize;
                    info!("device gpaddr is 0x{:x}", gpaddr);
                    let hpaddr = gpm.gpa_to_hpa(gpaddr);
                    unsafe {
                        *((self.base_vaddr + VIRTIO_DEVICE_LOW) as *mut u32) = hpaddr as u32;
                    }
                    unsafe {
                        *((self.base_vaddr + VIRTIO_DEVICE_HIGH) as *mut u32) =
                            (hpaddr >> 32) as u32;
                    }
                }
            }
            VIRTIO_DRIVER_LOW => {
                if let (Some(low), Some(high)) = (vqaddr.driver_gpa_low, vqaddr.driver_gpa_high) {
                    let gpaddr = ((high as usize) << 32) + low as usize;
                    info!("driver gpaddr is 0x{:x}", gpaddr);
                    let hpaddr = gpm.gpa_to_hpa(gpaddr);
                    unsafe {
                        *((self.base_vaddr + VIRTIO_DRIVER_LOW) as *mut u32) = hpaddr as u32;
                    }
                    unsafe {
                        *((self.base_vaddr + VIRTIO_DRIVER_HIGH) as *mut u32) =
                            (hpaddr >> 32) as u32;
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}

impl MMIODevice for Virtio {
    fn mem_range(&self) -> core::ops::Range<usize> {
        self.base_vaddr..self.base_vaddr + 0x4000
    }

    fn read(&self, addr: usize, access_size: u8) -> rvm::RvmResult<u32> {
        Ok(unsafe { *(addr as *const u32) })
    }

    fn write(
        &self,
        addr: usize,
        val: u32,
        access_size: u8,
        gpm: &GuestPhysMemorySet,
    ) -> rvm::RvmResult {
        // todo!()
        info!("virtio write addr {:x}, offset {:x}", addr, addr - self.base_vaddr);
        match (addr - self.base_vaddr) % VIRTIO_HEADER_EACH_SIZE {
            // todo: use marco
            // VIRTIO_NOTIFY => {

            // }
            VIRTIO_LEGACY_PFN => {
                let gpaddr = val as usize * PAGE_SIZE;
                if gpaddr == 0 {
                    loop { }
                    return Ok(());
                }
                info!("legacy gpaddr 0x{:x}", gpaddr);
                let hpaddr = gpm.gpa_to_hpa(gpaddr);
                info!("legacy gpaddr 0x{:x} hpaddr 0x{:x}", gpaddr, hpaddr);
                info!("legacy gpaddr next page 0x{:x} hpaddr 0x{:x}", gpaddr + 0x1000, gpm.gpa_to_hpa(gpaddr + 0x1000));
                let hpfn = hpaddr / PAGE_SIZE;
                self.virt_queue_addr.lock().legacy_vqaddr = hpaddr;
                unsafe { *(addr as *mut u32) = hpfn as u32; }
            }
            VIRTIO_DESC_LOW => {
                self.virt_queue_addr.lock().desc_gpa_low = Some(val);
                self.write_vqaddr(VIRTIO_DESC_LOW, gpm);
            }
            VIRTIO_DESC_HIGH => {
                self.virt_queue_addr.lock().desc_gpa_high = Some(val);
                self.write_vqaddr(VIRTIO_DESC_LOW, gpm);
            }
            VIRTIO_DEVICE_LOW => {
                self.virt_queue_addr.lock().device_gpa_low = Some(val);
                self.write_vqaddr(VIRTIO_DEVICE_LOW, gpm);
            }
            VIRTIO_DEVICE_HIGH => {
                self.virt_queue_addr.lock().device_gpa_high = Some(val);
                self.write_vqaddr(VIRTIO_DEVICE_LOW, gpm);
            }
            VIRTIO_DRIVER_LOW => {
                self.virt_queue_addr.lock().driver_gpa_low = Some(val);
                self.write_vqaddr(VIRTIO_DRIVER_LOW, gpm);
            }
            VIRTIO_DRIVER_HIGH => {
                self.virt_queue_addr.lock().driver_gpa_high = Some(val);
                self.write_vqaddr(VIRTIO_DRIVER_LOW, gpm);
            }
            _ => unsafe {
                *(addr as *mut u32) = val;
            },
        };
        Ok(())
    }
}
