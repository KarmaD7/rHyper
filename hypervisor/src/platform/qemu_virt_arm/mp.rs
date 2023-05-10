use crate::{
    config::{CPU_NUM, CPU_TO_VM, GUEST_ENTRIES},
    hv::gconfig::GUEST_ENTRY,
};

use super::psci::psci_start_cpu;

extern "C" {
    fn _start_secondary();
}

pub fn start_secondary_cpus(primary_cpu_id: usize) {
    let mut initialized = [false; CPU_NUM];
    let entry = _start_secondary as usize;
    let mut secondary_id = 0;
    initialized[CPU_TO_VM[primary_cpu_id]] = true;
    for i in 0..CPU_NUM {
        if i != primary_cpu_id {
            // let stack_top = unsafe { BOOT_STACK[i].as_ptr() as usize };
            // this is useless for psci.
            debug!("start secondary {}", i);
            start_secondary_cpu(i, entry, 0);
            if !initialized[CPU_TO_VM[i]] {
                unsafe {
                    GUEST_ENTRIES[i] = GUEST_ENTRY;
                }
            }
            initialized[CPU_TO_VM[i]] = true;
            // TODO: maintain mapping from vcpuid to cpuid
            secondary_id += 1;
        }
    }
}

fn start_secondary_cpu(cpu_id: usize, entry: usize, stack_top: usize) {
    psci_start_cpu(cpu_id, entry)
}
