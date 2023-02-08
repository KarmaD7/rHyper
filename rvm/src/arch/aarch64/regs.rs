#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct GeneralRegisters {
    pub x: [usize; 31],
}

macro_rules! save_regs_to_stack {
    () => {
        
    };
}

macro_rules! restore_regs_from_stack {
    () => {
        
    };
}

