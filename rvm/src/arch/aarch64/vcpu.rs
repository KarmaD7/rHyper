#![allow(dead_code)]
use core::{
    arch::asm,
    marker::PhantomData,
    mem::size_of,
};

use aarch64_cpu::asm::barrier;
use aarch64_cpu::registers::ESR_EL2::EC::Value;
use aarch64_cpu::registers::*;
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

use crate::{arch::aarch64::instructions, GuestPhysAddr, HostPhysAddr, RvmHal, RvmResult};

use super::{regs::GeneralRegisters, ArchPerCpuState};

#[repr(C)]
#[derive(Debug)]
pub struct ArmVcpu<H: RvmHal> {
    guest_regs: GeneralRegisters,
    guest_sp: u64,
    pub elr: u64,
    spsr: u64,
    host_stack_top: u64,
    pub cpu_id: u64,
    _phantom_data: PhantomData<H>,
}

impl<H: RvmHal> ArmVcpu<H> {
    pub(crate) fn new(
        _percpu: &ArchPerCpuState<H>,
        entry: GuestPhysAddr,
        npt_root: HostPhysAddr,
        cpu_id: u64,
    ) -> RvmResult<Self> {
        let vcpu = Self {
            guest_regs: GeneralRegisters::default(),
            guest_sp: 0,
            elr: entry as u64,
            spsr: (SPSR_EL2::M::EL1h
                + SPSR_EL2::D::Masked
                + SPSR_EL2::A::Masked
                + SPSR_EL2::I::Masked
                + SPSR_EL2::F::Masked)
                .into(),
            cpu_id,
            host_stack_top: 0,
            _phantom_data: PhantomData,
        };
        info!("npt root is {:x}.", npt_root);
        vcpu.setup(npt_root)?;
        info!("[RVM] created ArmVcpu");
        Ok(vcpu)
    }

    // #[repr(align(128))]
    pub fn run(&mut self) -> ! {
        unsafe { self.vm_launch() }
    }

    pub fn exit_info(&self) -> RvmResult<ArmExitInfo> {
        Ok(ArmExitInfo {
            exit_reason: ESR_EL2.read_as_enum(ESR_EL2::EC),
            guest_pc: ELR_EL2.get(),
        })
    }

    pub fn advance_pc(&mut self) -> RvmResult {
        let mut elr_el2 = ELR_EL2.get();
        elr_el2 += 4;
        ELR_EL2.set(elr_el2);
        Ok(())
    }

    pub fn cpu_id(&self) -> u64 {
        self.cpu_id
    }

    pub fn regs(&self) -> &GeneralRegisters {
        &self.guest_regs
    }

    pub fn regs_mut(&mut self) -> &mut GeneralRegisters {
        &mut self.guest_regs
    }


    pub fn set_page_table_root(&self, root: usize) {
        info!("TTBR0 set baddr {}", root);
        let attr0 = MAIR_EL1::Attr0_Device::nonGathering_nonReordering_EarlyWriteAck;
        // Normal memory
        let attr1 = MAIR_EL1::Attr1_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc
            + MAIR_EL1::Attr1_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc;
        MAIR_EL1.write(attr0 + attr1); // 0xff_04

        // Enable TTBR0 and TTBR1 walks, page size = 4K, vaddr size = 48 bits, paddr size = 40 bits.
        let tcr_flags0 = TCR_EL1::EPD0::EnableTTBR0Walks
            + TCR_EL1::TG0::KiB_4
            + TCR_EL1::SH0::Inner
            + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
            + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
            + TCR_EL1::T0SZ.val(16);
        TCR_EL1.write(TCR_EL1::IPS::Bits_44 + tcr_flags0);
        barrier::isb(barrier::SY);

        TTBR0_EL1.set_baddr(root as u64);
        instructions::flush_tlb_all();

        SCTLR_EL1.write(SCTLR_EL1::M::Enable + SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable);
    }

    fn setup(&self, npt_root: HostPhysAddr) -> RvmResult {
        // Disable EL1 timer traps and the timer offset.
        CNTHCTL_EL2.modify(CNTHCTL_EL2::EL1PCEN::SET + CNTHCTL_EL2::EL1PCTEN::SET);
        CNTVOFF_EL2.set(0);

        if !cfg!(feature = "linux") && cfg!(feature = "intr_emulate") {
            HCR_EL2.modify(HCR_EL2::IMO::SET);
        }

        let vtcr_flags = VTCR_EL2::TG0::Granule4KB
            + VTCR_EL2::SH0::Inner
            + VTCR_EL2::SL0.val(2)
            + VTCR_EL2::ORGN0::NormalWBRAWA
            + VTCR_EL2::IRGN0::NormalWBRAWA
            + VTCR_EL2::T0SZ.val(20);
        VTCR_EL2.write(VTCR_EL2::PS::PA_44B_16TB + vtcr_flags);
        barrier::isb(barrier::SY);

        VTTBR_EL2.set_baddr(npt_root as _);
        instructions::flush_tlb_all();

        Ok(())
    }

    #[naked]
    unsafe extern "C" fn vm_launch(&mut self) -> ! {
        asm!(
            "mov    x28, sp",
            "str    x28, [x0, {host_stack_top}]",   // save current SP to Vcpu::host_stack_top
            "mov    sp, x0",     // set SP to guest regs area
            restore_regs_from_stack!(),
            "eret",
            "bl    {failed}",
            host_stack_top = const size_of::<GeneralRegisters>() + 3 * size_of::<u64>(),
            failed = sym Self::vmentry_failed,
            options(noreturn),
        )
    }

    // This function is unused currently.
    // See HANDLE_SYNC in hypervisor/src/arch/aarch64/trap.S.
    #[naked]
    unsafe extern "C" fn vm_exit(&mut self) -> ! {
        asm!(
            save_regs_to_stack!(),
            "mov    x28, sp",                      // save temporary RSP to r15
            "mov    x0, sp",                      // set the first arg to &Vcpu
            "ldr    x29, [sp, {host_stack_top}]", // set RSP to Vcpu::host_stack_top
            "mov    sp, x29",
            "bl     {vmexit_handler}",              // call vmexit_handler
            "mov    sp, x28",                      // load temporary RSP from r15
            restore_regs_from_stack!(),
            "eret",
            "bl    {failed}",
            host_stack_top = const size_of::<GeneralRegisters>() + 3 * size_of::<u64>(),
            vmexit_handler = sym Self::vmexit_handler,
            failed = sym Self::vmentry_failed,
            options(noreturn),
        );
    }

    fn vmentry_failed() -> ! {
        panic!("vm entry failed")
    }

    fn vmexit_handler(&mut self) {
        H::vmexit_handler(self);
    }
}

pub type ArmExitReason = Option<Value>;

pub struct ArmExitInfo {
    pub exit_reason: ArmExitReason,
    guest_pc: u64,
}
