mod device_emu;
pub mod gconfig;
mod gpm;
mod hal;
mod vmexit;

use rvm::{GuestPhysAddr, HostPhysAddr, HostVirtAddr, MemFlags, RvmPerCpu, RvmResult};

use self::gconfig::*;
use self::gpm::{GuestMemoryRegion, GuestPhysMemorySet};
use self::hal::RvmHalImpl;
use crate::arch::instructions;
use crate::config::{CPU_NUM, CPU_TO_VM, VM_NUM};
use crate::mm::address::{align_up, phys_to_virt, virt_to_phys};

#[repr(align(4096))]
#[derive(Clone, Copy)]
struct AlignedMemory<const LEN: usize>([u8; LEN]);

static mut GUEST_PHYS_MEMORY: [AlignedMemory<GUEST_PHYS_MEMORY_SIZE>; VM_NUM] =
    [AlignedMemory([0; GUEST_PHYS_MEMORY_SIZE]); VM_NUM];

// pub static mut VIRTIO_HEADERS: AlignedMemory<VIRTIO_HEADER_TOTAL_SIZE> = AlignedMemory([0; VIRTIO_HEADER_TOTAL_SIZE]);

fn gpa_as_mut_ptr(guest_paddr: GuestPhysAddr, vm_id: usize) -> *mut u8 {
    let offset = unsafe { &GUEST_PHYS_MEMORY[vm_id] as *const _ as usize };
    let host_vaddr = guest_paddr - GUEST_PHYS_MEMORY_BASE + offset;
    // debug!("Host vaddr is {:x}.", host_vaddr);
    host_vaddr as *mut u8
}

fn load_guest_image(hpa: HostPhysAddr, load_gpa: GuestPhysAddr, size: usize, vm_id: usize) {
    debug!("loading guest image");
    let image_ptr = phys_to_virt(hpa) as *const u8;
    let image = unsafe { core::slice::from_raw_parts(image_ptr, size) };
    unsafe {
        core::slice::from_raw_parts_mut(gpa_as_mut_ptr(load_gpa, vm_id), size)
            .copy_from_slice(image)
        // core::slice::from_raw_parts_mut(load_gpa as *mut _, size).copy_from_slice(image)
    }
}

// fn setup_guest_page_table() {
//     use rvm::Stage1PTE;
//     let pt1 = unsafe { gpa_as_mut_ptr(GUEST_PT1) as *mut usize };
//     let pt2 = unsafe { gpa_as_mut_ptr(GUEST_PT2) as *mut usize };
//     unsafe {
//         *pt1 = Stage1PTE::new_table(
//             GUEST_PT2 as _,
//         ).0 as _;
//         *pt2 = Stage1PTE::new_page(
//             0,
//             MemFlags::READ | MemFlags::WRITE | MemFlags::EXECUTE,
//             true
//         ).0 as _;
//     }
//     // let pt2 = unsafe { &mut *(gpa_as_mut_ptr(GUEST_PT2) as *mut PageTable) };
//     // // identity mapping
//     // pt1[0].set_addr(
//     //     x86_64::PhysAddr::new(GUEST_PT2 as _),
//     //     PTF::PRESENT | PTF::WRITABLE,
//     // );
//     // pt2[0].set_addr(
//     //     x86_64::PhysAddr::new(0),
//     //     PTF::PRESENT | PTF::WRITABLE | PTF::HUGE_PAGE,
//     // );
// }

fn setup_gpm(cpu_id: usize) -> RvmResult<HostPhysAddr> {
    // setup_guest_page_table();
    // debug!("Set guest page table.");

    // copy guest code
    // unsafe {
    //     core::ptr::copy_nonoverlapping(
    //         test_guest as usize as *const u8,
    //         gpa_as_mut_ptr(GUEST_ENTRY),
    //         0x100,
    //     );
    // }

    info!("before setup gpm.");
    // create nested page table and add mapping
    let vm_id = CPU_TO_VM[cpu_id];
    info!("vm id {}.", vm_id);
    let mut gpms = GUEST_GPM.lock();
    // let guest_gpm = &mut gpm_guard[vm_id];
    if let Some(gpm) = &gpms[vm_id] {
        info!("initialized.");
        Ok(gpm.nest_page_table_root())
        // Err(rvm::RvmError::OutOfMemory)
    } else {
        load_guest_image(
            GUEST_IMAGE_PADDR + vm_id * GUEST_IMAGE_SIZE,
            GUEST_ENTRY,
            GUEST_IMAGE_SIZE,
            vm_id,
        );
        let mut gpm = GuestPhysMemorySet::new()?;
        let guest_memory_regions = [
            GuestMemoryRegion {
                // RAM
                gpa: GUEST_PHYS_MEMORY_BASE,
                hpa: virt_to_phys(gpa_as_mut_ptr(GUEST_PHYS_MEMORY_BASE, vm_id) as HostVirtAddr),
                size: GUEST_PHYS_MEMORY_SIZE,
                flags: MemFlags::READ | MemFlags::WRITE | MemFlags::EXECUTE,
            },
            GuestMemoryRegion {
                // pl011
                gpa: 0x0900_0000,
                hpa: 0x0900_0000,
                size: 0x1000,
                flags: MemFlags::READ | MemFlags::WRITE | MemFlags::DEVICE,
            },
            GuestMemoryRegion {
                // GICD
                gpa: 0x0800_0000,
                hpa: 0x0800_0000,
                size: 0x10000,
                flags: MemFlags::READ | MemFlags::WRITE | MemFlags::DEVICE,
            },
            GuestMemoryRegion {
                // GICC
                gpa: 0x0801_0000,
                hpa: 0x0801_0000,
                size: 0x10000,
                flags: MemFlags::READ | MemFlags::WRITE | MemFlags::DEVICE,
            },
            GuestMemoryRegion {
                // DTB
                gpa: GUEST_DTB_ADDR,
                hpa: GUEST_DTB.as_ptr() as usize,
                size: align_up(GUEST_DTB.len()),
                flags: MemFlags::READ | MemFlags::WRITE,
            },
            GuestMemoryRegion {
                // INITRAMFS
                gpa: GUEST_INITRAMFS_START,
                hpa: GUEST_INITRAMFS.as_ptr() as usize,
                size: align_up(GUEST_INITRAMFS.len()),
                flags: MemFlags::READ | MemFlags::WRITE,
            },
        ];
        for r in guest_memory_regions.into_iter() {
            info!("mapping");
            gpm.map_region(r.into())?;
        }
        let root = gpm.nest_page_table_root();
        gpms[vm_id] = Some(gpm);
        Ok(root)
    }
}

pub fn run(cpu_id: usize, entry: usize, psci_context: usize) -> ! {
    println!("Starting virtualization...");
    println!("Hardware support: {:?}", rvm::has_hardware_support());

    let mut percpu = RvmPerCpu::<RvmHalImpl>::new(cpu_id);
    info!("RVM new cpu");
    percpu.hardware_enable().unwrap();
    info!("RVM new cpu create");
    debug!("Vcpu Created.");

    if cpu_id == 0 {}
    let npt_root = setup_gpm(cpu_id).unwrap();
    // info!("{:#x?}", gpm);
    info!("Setup GPM.");

    let vcpu_id = cpu_id - CPU_TO_VM[cpu_id];
    info!("run vcpuid {}", vcpu_id);
    // TODO: move this to vcpu init
    unsafe {
        core::arch::asm!(
            "msr vmpidr_el2, {}", in(reg) vcpu_id
        );
    }
    let mut vcpu = percpu.create_vcpu(entry, npt_root).unwrap();
    // vcpu.set_page_table_root(GUEST_PT1);
    // vcpu.set_stack_pointer(GUEST_STACK_TOP);
    // info!("{:#x?}", vcpu);
    // TODO: move this to create_vcpu
    vcpu.regs_mut().x[0] = psci_context as u64;
    instructions::flush_tlb_all();
    println!("Running guest...");
    vcpu.run();
}

unsafe extern "C" fn test_guest() -> ! {
    for i in 0..100 {
        core::arch::asm!(
            "hvc #0",
            inout("x0") i => _,
            in("x1") 2,
            in("x2") 3,
            in("x3") 3,
            in("x4") 3,
        );
    }
    // // core::arch::asm!("mov qword ptr [$0xffff233], $2333"); // panic
    loop {}
}
