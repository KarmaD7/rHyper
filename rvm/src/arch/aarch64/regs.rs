#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct GeneralRegisters {
    pub x: [u64; 31],
}
// TODO
macro_rules! save_regs_to_stack {
    () => {
        "
        nop"
    };
}

macro_rules! restore_regs_from_stack {
    () => {
        "
        nop"
    };
}

