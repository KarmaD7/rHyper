use aarch64_cpu::registers::ESR_EL2;
use rvm::{RvmResult, RvmVcpu};
use rvm::arch::{ArmExitInfo, ArmExitReason};

use super::hal::RvmHalImpl;

type Vcpu = RvmVcpu<RvmHalImpl>;

fn handle_hypercall(vcpu: &mut Vcpu) -> RvmResult {
    // todo!();
    let regs = vcpu.regs();
    info!(
        "VM exit: VMCALL({:#x}): {:?}",
        regs.x[0],
        [regs.x[1], regs.x[2], regs.x[3], regs.x[4]]
    );
    // vcpu.advance_rip()?;
    Ok(())
}

#[no_mangle]
pub fn vmexit_handler(vcpu: &mut Vcpu) -> RvmResult {
    let exit_info = vcpu.exit_info()?;
    // debug!("VM exit: {:#x?}", exit_info);

    let res = match exit_info.exit_reason {
        Some(ESR_EL2::EC::Value::HVC64) => handle_hypercall(vcpu),
        _ => panic!(
            "Unhandled VM-Exit reason {:?}:\n{:#x?}",
            exit_info.exit_reason.unwrap() as u64, vcpu
        ),
    };

    if res.is_err() {
        panic!(
            "Failed to handle VM-exit {:?}:\n{:#x?}",
            exit_info.exit_reason.unwrap() as u64, vcpu
        );
    }

    Ok(())
}
