use core::arch::asm;

use aarch64_cpu::asm;
use spin::Mutex;
use alloc::{collections::BTreeMap, vec, vec::Vec};

use crate::{
    hv::{gconfig::VIRTIO_HEADER_EACH_SIZE, gpm::GuestPhysMemorySet},
    mm::PAGE_SIZE,
};

use super::MMIODevice;

const VIRTIO_QUEUE_SEL: usize = 0x30;
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
    legacy_vqaddr: BTreeMap<u32, usize>,
    queue_sel: u32,
    queue_size: BTreeMap<u32, u32>,
    last_notified_idx: BTreeMap<u32, u16>,
    translated: BTreeMap<u32, Vec<usize>>,
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
            legacy_vqaddr: BTreeMap::new(),
            queue_sel: 0,
            queue_size: BTreeMap::new(),
            last_notified_idx: BTreeMap::new(),
            translated: BTreeMap::new(),
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

    fn translate_desc_addr(&self, queue_sel: u32, gpm: &GuestPhysMemorySet) {
        // now only legacy devices are supported.
        // Note: in crate Virtio_drivers, unset desc buf will clear addr and len to 0.
        // Is it a specification of Virtio?
        // info!("notify queue sel {}", queue_sel);
        
        let mut queue_info = self.virt_queue_info.lock();
        unsafe {
            let desc_queue = core::slice::from_raw_parts_mut(
                queue_info.legacy_vqaddr[&queue_sel] as *mut Descriptor,
                queue_info.queue_size[&queue_sel] as usize,
            );
            let queue_size = queue_info.queue_size[&queue_sel];
            let hpaddrs = queue_info.translated.entry(queue_sel).or_insert(vec![0; queue_size as usize]);
            for i in 0..queue_size {
                if desc_queue[i as usize].len != 0 && desc_queue[i as usize].addr != 0 {
                    // valid
                    let gpa = desc_queue[i as usize].addr;
                    if hpaddrs[i as usize] == 0 || hpaddrs[i as usize] != desc_queue[i as usize].addr as usize {
                        // question: what if another desc's gpa equal to hpa?(to handle)
                        let hpaddr = gpm.gpa_to_hpa(gpa as usize);
                        hpaddrs[i as usize] = hpaddr;
                        desc_queue[i as usize].addr = hpaddr as u64;
                    }
                }
            }
        }
    }
}

impl MMIODevice for Virtio {
    fn mem_range(&self) -> core::ops::Range<usize> {
        self.base_vaddr..self.base_vaddr + 0x200
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
            VIRTIO_QUEUE_SEL => {
                let mut queue_info = self.virt_queue_info.lock();
                queue_info.queue_sel = val;
                queue_info.last_notified_idx.entry(val).or_insert(0);
                unsafe {
                    *(addr as *mut u32) = val;
                }
            }
            VIRTIO_QUEUE_SIZE => {
                let mut queue_info = self.virt_queue_info.lock();
                let idx = queue_info.queue_sel;
                queue_info.queue_size.insert(idx, val);
                trace!("Virt Queue Size: {}", val);
                unsafe {
                    *(addr as *mut u32) = val;
                }
            }
            VIRTIO_NOTIFY => {
                trace!("notify");
                self.translate_desc_addr(val, gpm);
                unsafe {
                    *(addr as *mut u32) = val;
                }
            }
            VIRTIO_LEGACY_PFN => {
                let gpaddr = val as usize * PAGE_SIZE;
                info!("legacy gpaddr 0x{:x}", gpaddr);
                let hpaddr = gpm.gpa_to_hpa(gpaddr);
                info!("legacy gpaddr 0x{:x} hpaddr 0x{:x}", gpaddr, hpaddr);
                trace!(
                    "legacy gpaddr next page 0x{:x} hpaddr 0x{:x}",
                    gpaddr + 0x1000,
                    gpm.gpa_to_hpa(gpaddr + 0x1000)
                );
                let hpfn = hpaddr / PAGE_SIZE;
                let mut queue_info = self.virt_queue_info.lock();
                let idx = queue_info.queue_sel;
                info!("Write {}'s pfn", idx);
                queue_info.legacy_vqaddr.insert(idx, hpaddr);
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
