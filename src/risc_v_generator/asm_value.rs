use super::program_manager::*;

pub enum AsmValue {
    Global(String),
    LocalVar(ValueSlot),
    Const(i32),
    FuncArg(usize),
    Void,
}

impl AsmValue {
    pub fn is_ptr(&self) -> bool {
        match self {
            Self::Global(_) => true,
            Self::LocalVar(slot) => slot.is_ptr(),
            Self::Const(_) => false,
            Self::FuncArg(_) => true,
            Self::Void => false,
        }
    }
}