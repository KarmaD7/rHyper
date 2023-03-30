use core::arch::asm;

pub const PSCI_CPU_ON: usize = 0x84000003;
pub const PSCI_CPU_OFF: usize = 0x84000008;

fn psci_smc_call(func: usize, args0: usize, args1: usize, args2: usize) -> usize {
    let ret;
    unsafe {
        asm!("smc #0", 
        inlateout("x0") func => ret,
        in("x1") args0,
        in("x2") args1,
        in("x3") args2)
    }
    ret
}

pub fn psci_start_cpu(cpuid: usize, entry: usize) {
    info!("trying to start cpu {}.", cpuid);
    assert_eq!(psci_smc_call(PSCI_CPU_ON, cpuid, entry, 0), 0);
    info!("cpu {} started.", cpuid)
}
