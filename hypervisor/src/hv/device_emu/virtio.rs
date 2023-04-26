use core::arch::asm;

use aarch64_cpu::asm;
use spin::Mutex;

use crate::{
    hv::{gconfig::VIRTIO_HEADER_EACH_SIZE, gpm::GuestPhysMemorySet},
    mm::PAGE_SIZE,
};

use super::MMIODevice;

const VIRTIO_QUEUE_SIZE: usize = 0x38;
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
    virt_queue_info: Mutex<VirtQueueInfo>,
}

bitflags::bitflags! {
    /// Descriptor flags
    struct DescFlags: u16 {
        const NEXT = 1 << 0;
        const WRITE = 1 << 1;
        const INDIRECT = 1 << 2;
    }
}

#[repr(C, align(16))]
#[derive(Clone, Debug)]
pub struct Descriptor {
    addr: u64,
    len: u32,
    flags: DescFlags,
    next: u16,
}

#[derive(Default)]
struct VirtQueueInfo {
    desc_gpa_low: Option<u32>,
    desc_gpa_high: Option<u32>,
    driver_gpa_low: Option<u32>,
    driver_gpa_high: Option<u32>,
    device_gpa_low: Option<u32>,
    device_gpa_high: Option<u32>,
    legacy_vqaddr: usize,
    queue_size: u32,
    last_notified_idx: u16,
}

impl VirtQueueInfo {
    pub const fn new() -> Self {
        Self {
            desc_gpa_low: None,
            desc_gpa_high: None,
            driver_gpa_low: None,
            driver_gpa_high: None,
            device_gpa_low: None,
            device_gpa_high: None,
            legacy_vqaddr: 0,
            queue_size: 0,
            last_notified_idx: 0,
        }
    }
}

impl Virtio {
    pub const fn new(base_vaddr: usize) -> Self {
        Self {
            base_vaddr,
            virt_queue_info: Mutex::new(VirtQueueInfo::new()),
        }
    }

    fn write_vqaddr(&self, offset: usize, gpm: &GuestPhysMemorySet) {
        //todo: use macro
        let vqaddr = self.virt_queue_info.lock();
        match offset {
            VIRTIO_DESC_LOW => {
                if let (Some(low), Some(high)) = (vqaddr.desc_gpa_low, vqaddr.desc_gpa_high) {
                    let gpaddr = ((high as usize) << 32) + low as usize;
                    trace!("desc gpaddr is 0x{:x}", gpaddr);
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
                    trace!("device gpaddr is 0x{:x}", gpaddr);
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
                    trace!("driver gpaddr is 0x{:x}", gpaddr);
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

    fn translate_desc_addr(&self, gpm: &GuestPhysMemorySet) {
        // now only legacy devices are supported.
        let mut queue_info = self.virt_queue_info.lock();
        unsafe {
            let desc_queue = core::slice::from_raw_parts_mut(
                queue_info.legacy_vqaddr as *mut Descriptor,
                queue_info.queue_size as usize,
            );
            loop {
                let idx = queue_info.last_notified_idx as u32 & (queue_info.queue_size - 1);
                let gpaddr = desc_queue[idx as usize].addr;
                info!(
                    "to translate descqueue idx {} from gpa 0x{:x}",
                    idx, gpaddr
                );
                let flags = desc_queue[idx as usize].flags;
                for i in 0..3 {
                    info!(
                        "descqueue idx {} gpa 0x{:x} flag {:?}",
                        i, desc_queue[i].addr, desc_queue[i].flags
                    );
                }
                let hpaddr = gpm.gpa_to_hpa(gpaddr as usize);
                info!(
                    "Translating descqueue idx {} flags {:?} from gpa 0x{:x} to hpa 0x{:x}",
                    idx, flags, gpaddr, hpaddr
                );
                desc_queue[idx as usize].addr = hpaddr as u64;
                queue_info.last_notified_idx += 1;
                if !flags.contains(DescFlags::NEXT) {
                    // If the guest os use non-blocking read/write, we need to maintaining last_notified_idx by looking used and avail.
                    // As now arceos just use blocking read/write, we simply set last_notified_idx to 0, for all the elements 
                    // would have been poped before next request.
                    queue_info.last_notified_idx = 0;
                    break;
                }
            }
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
        trace!(
            "virtio write addr {:x}, offset {:x}",
            addr,
            addr - self.base_vaddr
        );
        match (addr - self.base_vaddr) % VIRTIO_HEADER_EACH_SIZE {
            // todo: use marco
            VIRTIO_QUEUE_SIZE => {
                self.virt_queue_info.lock().queue_size = val;
                trace!("Virt Queue Size: {}", val);
                unsafe {
                    *(addr as *mut u32) = val;
                }
            }
            VIRTIO_NOTIFY => {
                trace!("notify");
                self.translate_desc_addr(gpm);
                unsafe {
                    *(addr as *mut u32) = val;
                }
            }
            VIRTIO_LEGACY_PFN => {
                let gpaddr = val as usize * PAGE_SIZE;
                trace!("legacy gpaddr 0x{:x}", gpaddr);
                let hpaddr = gpm.gpa_to_hpa(gpaddr);
                trace!("legacy gpaddr 0x{:x} hpaddr 0x{:x}", gpaddr, hpaddr);
                trace!(
                    "legacy gpaddr next page 0x{:x} hpaddr 0x{:x}",
                    gpaddr + 0x1000,
                    gpm.gpa_to_hpa(gpaddr + 0x1000)
                );
                let hpfn = hpaddr / PAGE_SIZE;
                self.virt_queue_info.lock().legacy_vqaddr = hpaddr;
                unsafe {
                    *(addr as *mut u32) = hpfn as u32;
                }
            }
            VIRTIO_DESC_LOW => {
                self.virt_queue_info.lock().desc_gpa_low = Some(val);
                self.write_vqaddr(VIRTIO_DESC_LOW, gpm);
            }
            VIRTIO_DESC_HIGH => {
                self.virt_queue_info.lock().desc_gpa_high = Some(val);
                self.write_vqaddr(VIRTIO_DESC_LOW, gpm);
            }
            VIRTIO_DEVICE_LOW => {
                self.virt_queue_info.lock().device_gpa_low = Some(val);
                self.write_vqaddr(VIRTIO_DEVICE_LOW, gpm);
            }
            VIRTIO_DEVICE_HIGH => {
                self.virt_queue_info.lock().device_gpa_high = Some(val);
                self.write_vqaddr(VIRTIO_DEVICE_LOW, gpm);
            }
            VIRTIO_DRIVER_LOW => {
                self.virt_queue_info.lock().driver_gpa_low = Some(val);
                self.write_vqaddr(VIRTIO_DRIVER_LOW, gpm);
            }
            VIRTIO_DRIVER_HIGH => {
                self.virt_queue_info.lock().driver_gpa_high = Some(val);
                self.write_vqaddr(VIRTIO_DRIVER_LOW, gpm);
            }
            _ => unsafe {
                *(addr as *mut u32) = val;
            },
        };
        Ok(())
    }
}
