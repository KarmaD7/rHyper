use rvm::{RvmResult, RvmVcpu};

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
    vcpu.advance_rip()?;
    Ok(())
}

fn vmexit_handler(vcpu: &mut Vcpu) -> RvmResult {
    let exit_info = vcpu.exit_info()?;
    debug!("VM exit: {:#x?}", exit_info);

    let res = match exit_info.exit_reason {
        VmxExitReason::VMCALL => handle_hypercall(vcpu),
        _ => panic!(
            "Unhandled VM-Exit reason {:?}:\n{:#x?}",
            exit_info.exit_reason, vcpu
        ),
    };

    if res.is_err() {
        panic!(
            "Failed to handle VM-exit {:?}:\n{:#x?}",
            exit_info.exit_reason, vcpu
        );
    }

    Ok(())
}
