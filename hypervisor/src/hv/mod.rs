use rvm::RvmPerCpu;
use self::hal::RvmHalImpl;

mod hal;
mod vmexit;

pub fn run() {
    println!("Starting virtualization...");
    println!("Hardware support: {:?}", rvm::has_hardware_support());

    let mut percpu = RvmPerCpu::<RvmHalImpl>::new(0);
    percpu.hardware_enable().unwrap();

    let mut vcpu = percpu.create_vcpu(test_guest as usize).unwrap();
    info!("{:#x?}", vcpu);
    println!("Running guest...");
    vcpu.run();
}

#[naked]
unsafe extern "C" fn test_guest() -> ! { 
    core::arch::asm!(
        "
        mov     x0, 0
        mov     x1, 2
        mov     x2, 3
        mov     x3, 3
        mov     x4, 3
    2:
        hvc     #0
        add     x0, x0, 1
        b       2b",
        options(noreturn),
    );
}