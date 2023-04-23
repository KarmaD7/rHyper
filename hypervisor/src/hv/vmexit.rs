use aarch64_cpu::registers::{ESR_EL2, FAR_EL2};
use rvm::{RvmResult, RvmVcpu};
use tock_registers::interfaces::Readable;

use crate::{
    device::{gicv2::deactivate_irq, inject_irq, pending_irq},
    hv::device_emu::all_virt_devices,
};

use super::{
    gconfig::{CPU_PARTITION, GUEST_GPM},
    hal::RvmHalImpl,
};

pub type Vcpu = RvmVcpu<RvmHalImpl>;

fn handle_hypercall(vcpu: &mut Vcpu) -> RvmResult {
    let regs = vcpu.regs();
    info!(
        "VM exit: VMCALL({:#x}): {:?}",
        regs.x[0],
        [regs.x[1], regs.x[2], regs.x[3], regs.x[4]]
    );
    match regs.x[0] {
        PSCI_CPU_OFF => loop {},
        _ => {}
    }
    Ok(())
}

fn handle_iabt(vcpu: &mut Vcpu) -> RvmResult {
    // todo!();
    // info!("VTTBR_EL2: {:x}", VTTBR_EL2.get());
    // Ok(())
    error!("Instruction abort!!!");
    Err(rvm::RvmError::ResourceBusy)
}

#[no_mangle]
fn handle_dabt(vcpu: &mut Vcpu) -> RvmResult {
    // we need to add HPFAR_EL2 to aarch64_cpu
    // FAR_EL2 val is not correct, we use it temporarily
    // if vcpu.cpu_id != 0 {
    //     info!("cpu {} handling dabt", vcpu.cpu_id);
    // }
    let fault_vaddr = FAR_EL2.get() & 0xffff_ffff_ffff;
    // info!("handling dabt, fault addr 0x{:x}", fault_vaddr);
    let iss = ESR_EL2.read(ESR_EL2::ISS);
    let isv = iss >> 24;
    let sas = iss >> 22 & 0x3;
    let sse = iss >> 21 & 0x1;
    let srt = iss >> 16 & 0x1f;
    let ea = iss >> 9 & 0x1;
    let cm = iss >> 8 & 0x1;
    let s1ptw = iss >> 7 & 0x1;
    let is_write = iss >> 6 & 0x1;
    let size = 1 << sas;
    let val = if is_write == 1 && srt != 31 {
        vcpu.regs().x[srt as usize]
    } else {
        0
    };

    if let Some(dev) = all_virt_devices().find_mmio_device(fault_vaddr as usize) {
        // info!("iss {:x} is write {}", iss, is_write);
        if is_write == 1 {
            let cpu_id = vcpu.cpu_id;
            let guest_id = CPU_PARTITION[cpu_id as usize];
            let mut gpms = GUEST_GPM.lock();
            // let guest_gpm = &mut gpm_guard[guest_id];
            if let Some(gpm) = &gpms[guest_id] {
                dev.write(fault_vaddr as usize, val as u32, size, gpm)?;
            } else {
                error!("Guest Without GPM!");
            };
        } else {
            vcpu.regs_mut().x[srt as usize] = dev.read(fault_vaddr as usize, size)? as u64;
            // info!("elr {:x} srt {:x} read val {:x}", vcpu.elr, srt, vcpu.regs().x[srt as usize]);
        }
        vcpu.advance_rip()?;
        Ok(())
    } else {
        Err(rvm::RvmError::OutOfMemory)
    }
}

#[no_mangle]
pub fn vmexit_handler(vcpu: &mut Vcpu) -> RvmResult {
    let exit_info = vcpu.exit_info()?;
    // if vcpu.cpu_id != 0 {
    // println!("cpu {} exit", vcpu.cpu_id);
    // }
    // debug!("VM exit: {:#x?}", exit_info);

    let res = match exit_info.exit_reason {
        Some(ESR_EL2::EC::Value::HVC64) => handle_hypercall(vcpu),
        Some(ESR_EL2::EC::Value::InstrAbortLowerEL) => handle_iabt(vcpu),
        Some(ESR_EL2::EC::Value::InstrAbortCurrentEL) => handle_iabt(vcpu),
        Some(ESR_EL2::EC::Value::DataAbortLowerEL) => handle_dabt(vcpu),
        Some(ESR_EL2::EC::Value::DataAbortCurrentEL) => handle_dabt(vcpu),
        _ => panic!(
            "Unhandled VM-Exit reason {:?}:\n{:#x?}",
            exit_info.exit_reason.unwrap() as u64,
            vcpu
        ),
    };

    if res.is_err() {
        panic!(
            "Failed to handle VM-exit {:?}:\n{:#x?}",
            exit_info.exit_reason.unwrap() as u64,
            vcpu
        );
    }

    Ok(())
}

#[no_mangle]
pub fn irq_handler() -> RvmResult {
    // info!("IRQ routed to EL2");
    if let Some(irq_id) = pending_irq() {
        if irq_id != 30 {
            info!("IRQ {} routed to EL2", irq_id);
        }
        deactivate_irq(irq_id);
        inject_irq(irq_id);
    }

    Ok(())
    // // let irq_number =
    // todo!()
}
