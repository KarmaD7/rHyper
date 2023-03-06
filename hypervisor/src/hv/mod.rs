mod gconfig;
mod gpm;
mod hal;
mod vmexit;

use rvm::{GenericPTE, GuestPhysAddr, HostPhysAddr, HostVirtAddr, MemFlags, RvmPerCpu, RvmResult};

use self::gconfig::*;
use self::gpm::{GuestMemoryRegion, GuestPhysMemorySet};
use self::hal::RvmHalImpl;
use crate::arch::instructions;
use crate::mm::address::{phys_to_virt, virt_to_phys};

#[repr(align(4096))]
struct AlignedMemory<const LEN: usize>([u8; LEN]);

static mut GUEST_PHYS_MEMORY: AlignedMemory<GUEST_PHYS_MEMORY_SIZE> =
    AlignedMemory([0; GUEST_PHYS_MEMORY_SIZE]);

fn gpa_as_mut_ptr(guest_paddr: GuestPhysAddr) -> *mut u8 {
    let offset = unsafe { &GUEST_PHYS_MEMORY as *const _ as usize };
    let host_vaddr = guest_paddr - GUEST_PHYS_MEMORY_BASE + offset;
    info!("Host vaddr is {:x}.", host_vaddr);
    host_vaddr as *mut u8
}

fn load_guest_image(hpa: HostPhysAddr, load_gpa: GuestPhysAddr, size: usize) {
    info!("loading guest image");
    let image_ptr = phys_to_virt(hpa) as *const u8;
    let image = unsafe { core::slice::from_raw_parts(image_ptr, size) };
    unsafe {
        core::slice::from_raw_parts_mut(gpa_as_mut_ptr(load_gpa), size).copy_from_slice(image)
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

fn setup_gpm() -> RvmResult<GuestPhysMemorySet> {
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
    load_guest_image(GUEST_IMAGE_PADDR, GUEST_ENTRY, GUEST_IMAGE_SIZE);

    debug!("before setup gpm.");
    // create nested page table and add mapping
    let mut gpm = GuestPhysMemorySet::new()?;
    debug!("HPA: {:x}", gpa_as_mut_ptr(0) as HostVirtAddr);
    let guest_memory_regions = [
        GuestMemoryRegion {
            // RAM
            gpa: GUEST_PHYS_MEMORY_BASE,
            hpa: gpa_as_mut_ptr(GUEST_PHYS_MEMORY_BASE) as HostVirtAddr,
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
            // GICv2
            gpa: 0x0800_0000,
            hpa: 0x0800_0000,
            size: 0x20000,
            flags: MemFlags::READ | MemFlags::WRITE | MemFlags::DEVICE,
        },
    ];
    for r in guest_memory_regions.into_iter() {
        info!("mapping");
        gpm.map_region(r.into())?;
    }
    Ok(gpm)
}

pub fn run() -> ! {
    println!("Starting virtualization...");
    println!("Hardware support: {:?}", rvm::has_hardware_support());

    let mut percpu = RvmPerCpu::<RvmHalImpl>::new(0);
    percpu.hardware_enable().unwrap();
    debug!("Vcpu Created.");

    let gpm = setup_gpm().unwrap();
    // info!("{:#x?}", gpm);
    debug!("Setup GPM.");

    let mut vcpu = percpu
        .create_vcpu(GUEST_ENTRY, gpm.nest_page_table_root())
        .unwrap();
    // vcpu.set_page_table_root(GUEST_PT1);
    // vcpu.set_stack_pointer(GUEST_STACK_TOP);
    // info!("{:#x?}", vcpu);
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
