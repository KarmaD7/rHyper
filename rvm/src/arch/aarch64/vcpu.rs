use core::{marker::PhantomData, arch::asm, mem::size_of};

use aarch64_cpu::registers::{ELR_EL2, VBAR_EL2};
use tock_registers::interfaces::{Readable, Writeable};

use crate::{GuestPhysAddr, RvmHal, RvmResult};

use super::{regs::GeneralRegisters, ArchPerCpuState};

#[repr(C)]
#[derive(Debug)]
pub struct ArmVcpu<H: RvmHal> {
    guest_regs: GeneralRegisters,
    host_stack_top: u64,
    _phantom_data: PhantomData<H>,
}

impl<H: RvmHal> ArmVcpu<H> {
    pub(crate) fn new(_percpu: &ArchPerCpuState<H>, entry: GuestPhysAddr) -> RvmResult<Self> {
        let mut vcpu = Self {
            guest_regs: GeneralRegisters::default(),
            host_stack_top: 0,
            _phantom_data: PhantomData,
        };
        vcpu.setup(entry)?;
        info!("[RVM] created ArmVcpu");
        Ok(vcpu)
    }

    pub fn run(&self) -> () {
        
    }

    pub fn exit_info(&self) -> RvmResult<ArmExitInfo> {
        Ok(ArmExitInfo {
            exit_reason: ArmExitReason::HLT,
            guest_pc: 0,
        })
    }

    pub fn regs(&self) -> &GeneralRegisters {
        &self.guest_regs
    }

    pub fn regs_mut(&mut self) -> &mut GeneralRegisters {
        &mut self.guest_regs
    }

    pub fn advance_rip(&self) -> RvmResult {
        Ok(ELR_EL2.set(ELR_EL2.get() + 4))
    }

    fn setup(&mut self, entry: GuestPhysAddr) -> RvmResult {
        VBAR_EL2.set(entry as u64);
        Ok(())
    }

    #[naked]
    unsafe extern "C" fn vmx_launch(&mut self) -> ! {
        asm!(
            "mov    [rdi + {host_stack_top}], rsp", // save current RSP to Vcpu::host_stack_top
            "mov    rsp, rdi",                      // set RSP to guest regs area
            restore_regs_from_stack!(),
            "vmlaunch",
            "jmp    {failed}",
            host_stack_top = const size_of::<GeneralRegisters>(),
            failed = sym Self::vmentry_failed,
            options(noreturn),
        )
    }

    #[naked]
    unsafe extern "C" fn vmx_exit(&mut self) -> ! {
        asm!(
            save_regs_to_stack!(),
            "mov    r15, rsp",                      // save temporary RSP to r15
            "mov    rdi, rsp",                      // set the first arg to &Vcpu
            "mov    rsp, [rsp + {host_stack_top}]", // set RSP to Vcpu::host_stack_top
            "call   {vmexit_handler}",              // call vmexit_handler
            "mov    rsp, r15",                      // load temporary RSP from r15
            restore_regs_from_stack!(),
            "vmresume",
            "jmp    {failed}",
            host_stack_top = const size_of::<GeneralRegisters>(),
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
    EXCEPTION_NMI = 0,
    EXTERNAL_INTERRUPT = 1,
    TRIPLE_FAULT = 2,
    INIT = 3,
    SIPI = 4,
    SMI = 5,
    OTHER_SMI = 6,
    INTERRUPT_WINDOW = 7,
    NMI_WINDOW = 8,
    TASK_SWITCH = 9,
    CPUID = 10,
    GETSEC = 11,
    HLT = 12,
    INVD = 13,
    INVLPG = 14,
    RDPMC = 15,
    RDTSC = 16,
    RSM = 17,
    VMCALL = 18,
    VMCLEAR = 19,
    VMLAUNCH = 20,
    VMPTRLD = 21,
    VMPTRST = 22,
    VMREAD = 23,
    VMRESUME = 24,
    VMWRITE = 25,
    VMOFF = 26,
    VMON = 27,
    CR_ACCESS = 28,
    DR_ACCESS = 29,
    IO_INSTRUCTION = 30,
    MSR_READ = 31,
    MSR_WRITE = 32,
    INVALID_GUEST_STATE = 33,
    MSR_LOAD_FAIL = 34,
    MWAIT_INSTRUCTION = 36,
    MONITOR_TRAP_FLAG = 37,
    MONITOR_INSTRUCTION = 39,
    PAUSE_INSTRUCTION = 40,
    MCE_DURING_VMENTRY = 41,
    TPR_BELOW_THRESHOLD = 43,
    APIC_ACCESS = 44,
    VIRTUALIZED_EOI = 45,
    GDTR_IDTR = 46,
    LDTR_TR = 47,
    EPT_VIOLATION = 48,
    EPT_MISCONFIG = 49,
    INVEPT = 50,
    RDTSCP = 51,
    PREEMPTION_TIMER = 52,
    INVVPID = 53,
    WBINVD = 54,
    XSETBV = 55,
    APIC_WRITE = 56,
    RDRAND = 57,
    INVPCID = 58,
    VMFUNC = 59,
    ENCLS = 60,
    RDSEED = 61,
    PML_FULL = 62,
    XSAVES = 63,
    XRSTORS = 64,
    PCONFIG = 65,
    SPP_EVENT = 66,
    UMWAIT = 67,
    TPAUSE = 68,
    LOADIWKEY = 69,
}
}

#[derive(Debug)]
pub struct ArmExitInfo {
    pub exit_reason: ArmExitReason,

    guest_pc: u64,
}
