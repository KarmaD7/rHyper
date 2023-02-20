use core::{marker::PhantomData, arch::asm, mem::size_of};

use aarch64_cpu::registers::{ELR_EL2, VBAR_EL2, SPSR_EL2, SP_EL1};
use tock_registers::interfaces::{Readable, Writeable};

use crate::{GuestPhysAddr, RvmHal, RvmResult};

use super::{regs::GeneralRegisters, ArchPerCpuState};

#[repr(C)]
#[derive(Debug)]
pub struct ArmVcpu<H: RvmHal> {
    guest_regs: GeneralRegisters,
    guest_sp: u64,
    elr: u64,
    spsr: u64,
    host_stack_top: u64,
    _phantom_data: PhantomData<H>,
}

impl<H: RvmHal> ArmVcpu<H> {
    pub(crate) fn new(_percpu: &ArchPerCpuState<H>, entry: GuestPhysAddr) -> RvmResult<Self> {
        let mut vcpu = Self {
            host_stack_top: 0,
            guest_regs: GeneralRegisters::default(),
            guest_sp: 0, // todo
            elr: entry as u64,
            spsr: (SPSR_EL2::M::EL1h
            + SPSR_EL2::D::Masked
            + SPSR_EL2::A::Masked
            + SPSR_EL2::I::Masked
            + SPSR_EL2::F::Masked).into(),
            _phantom_data: PhantomData,
        };
        vcpu.setup()?;
        info!("[RVM] created ArmVcpu");
        Ok(vcpu)
    }

    // #[repr(align(128))]
    pub fn run(&mut self) -> ! {
        self.setup().unwrap();
        unsafe { self.vmx_launch() }
    }

    pub fn exit_info(&self) -> RvmResult<ArmExitInfo> {
        Ok(ArmExitInfo {
            exit_reason: ArmExitReason::HVC,
            guest_pc: 0,
        })
    }

    pub fn regs(&self) -> &GeneralRegisters {
        &self.guest_regs
    }

    pub fn regs_mut(&mut self) -> &mut GeneralRegisters {
        &mut self.guest_regs
    }

    pub fn advance_rip(&mut self) -> RvmResult {
        self.elr += 4;
        Ok(())
    }

    fn setup(&self) -> RvmResult {
        VBAR_EL2.set(Self::vmx_exit as u64 + 8 - 0x400);
        Ok(())
    }

    #[naked]
    unsafe extern "C" fn vmx_launch(&mut self) -> ! {
        asm!(
            "mov    x30, sp",
            "str    x30, [x0, {host_stack_top}]",   // save current RSP to Vcpu::host_stack_top
            "mov    sp, x0",     // set RSP to guest regs area
            restore_regs_from_stack!(),
            "eret",
            "bl    {failed}",
            host_stack_top = const size_of::<GeneralRegisters>() + 3 * size_of::<u64>(),
            failed = sym Self::vmentry_failed,
            options(noreturn),
        )
    }

    #[naked]
    unsafe extern "C" fn vmx_exit(&mut self) -> ! {
        asm!(
            "add    x28, x28, 12",
            "add    x27, x27, 13",
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

numeric_enum_macro::numeric_enum! {
#[repr(u32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[allow(non_camel_case_types)]
/// VMX basic exit reasons. (SDM Vol. 3D, Appendix C)
pub enum ArmExitReason {
    HVC = 16
    // EXCEPTION_NMI = 0,
    // EXTERNAL_INTERRUPT = 1,
    // TRIPLE_FAULT = 2,
    // INIT = 3,
    // SIPI = 4,
    // SMI = 5,
    // OTHER_SMI = 6,
    // INTERRUPT_WINDOW = 7,
    // NMI_WINDOW = 8,
    // TASK_SWITCH = 9,
    // CPUID = 10,
    // GETSEC = 11,
    // HLT = 12,
    // INVD = 13,
    // INVLPG = 14,
    // RDPMC = 15,
    // HVC = 16,
    // RSM = 17,
    // VMCALL = 18,
    // VMCLEAR = 19,
    // VMLAUNCH = 20,
    // VMPTRLD = 21,
    // VMPTRST = 22,
    // VMREAD = 23,
    // VMRESUME = 24,
    // VMWRITE = 25,
    // VMOFF = 26,
    // VMON = 27,
    // CR_ACCESS = 28,
    // DR_ACCESS = 29,
    // IO_INSTRUCTION = 30,
    // MSR_READ = 31,
    // MSR_WRITE = 32,
    // INVALID_GUEST_STATE = 33,
    // MSR_LOAD_FAIL = 34,
    // MWAIT_INSTRUCTION = 36,
    // MONITOR_TRAP_FLAG = 37,
    // MONITOR_INSTRUCTION = 39,
    // PAUSE_INSTRUCTION = 40,
    // MCE_DURING_VMENTRY = 41,
    // TPR_BELOW_THRESHOLD = 43,
    // APIC_ACCESS = 44,
    // VIRTUALIZED_EOI = 45,
    // GDTR_IDTR = 46,
    // LDTR_TR = 47,
    // EPT_VIOLATION = 48,
    // EPT_MISCONFIG = 49,
    // INVEPT = 50,
    // RDTSCP = 51,
    // PREEMPTION_TIMER = 52,
    // INVVPID = 53,
    // WBINVD = 54,
    // XSETBV = 55,
    // APIC_WRITE = 56,
    // RDRAND = 57,
    // INVPCID = 58,
    // VMFUNC = 59,
    // ENCLS = 60,
    // RDSEED = 61,
    // PML_FULL = 62,
    // XSAVES = 63,
    // XRSTORS = 64,
    // PCONFIG = 65,
    // SPP_EVENT = 66,
    // UMWAIT = 67,
    // TPAUSE = 68,
    // LOADIWKEY = 69,
}
}

#[derive(Debug)]
pub struct ArmExitInfo {
    pub exit_reason: ArmExitReason,

    guest_pc: u64,
}
