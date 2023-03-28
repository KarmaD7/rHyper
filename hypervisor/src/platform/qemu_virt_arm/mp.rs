use crate::config::CPU_NUM;

use super::psci::psci_start_cpu;

extern "C" {
    fn _start_secondary();
}

pub fn start_secondary_cpus(primary_cpu_id: usize) {
    let entry = _start_secondary as usize;
    let mut secondary_id = 0;
    for i in 0..CPU_NUM {
        if i != primary_cpu_id {
            // let stack_top = unsafe { BOOT_STACK[i].as_ptr() as usize };
            // this is useless for psci.
            start_secondary_cpu(i, entry, 0);
            secondary_id += 1;
        }
    }
}

fn start_secondary_cpu(cpu_id: usize, entry: usize, stack_top: usize) {
    psci_start_cpu(cpu_id, entry)
}
