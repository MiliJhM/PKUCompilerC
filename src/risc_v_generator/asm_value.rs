use super::{program_manager::*, asm_generator::Writer};
use std::io::Result;
pub enum AsmValue {
    Global(String),
    LocalVar(ValueSlot),
    Const(i32),
    FuncArg(usize),
    Void,
}

impl<'file> AsmValue {
    pub fn is_ptr(&self) -> bool {
        match self {
            Self::Global(_) => false,
            Self::LocalVar(slot) => slot.is_ptr(),
            Self::Const(_) => false,
            Self::FuncArg(_) => false,
            Self::Void => false,
        }
    }

    pub fn normal_to_reg(&self, f: &'file mut Writer, reg: &'static str) -> Result<()> {
        let res = f.reg_temp;
        f.update_temp_reg(reg);
        match self{
            Self::Global(name) => {
                f.la(reg, name.as_str());
                f.lw(reg, reg, 0);
            },
            Self::LocalVar(slot) => {
                assert!(slot.stack.is_some());
                f.lw(reg, "sp", slot.stackslot_offset().unwrap() as i32);
            }
            Self::Const(val) => {
                f.li(reg, *val);
            }
            _ => unreachable!()
        };
        f.update_temp_reg(res);
        Ok(())
    }

    pub fn load_addr_to_reg(&self, f: &'file mut Writer, reg: &'static str) -> Result<()> {
        let res = f.reg_temp;
        f.update_temp_reg(reg);

        match self{
            Self::Global(name) => {
                f.la(reg, name.as_str());
            },
            Self::LocalVar(slot) => {
                assert!(slot.stack.is_some());
                f.addi(reg, "sp", slot.stackslot_offset().unwrap() as i32);
            }
            _ => unreachable!()
        };
        f.update_temp_reg(res);
        Ok(())
    }

    pub fn arg_to_reg(&self, f: &'file mut Writer, reg: &'static str, spoff:usize) -> Result<()> {
        let res = f.reg_temp;
        f.update_temp_reg(reg);

        match self{
            Self::FuncArg(index) => {
                if *index < 8 {
                    f.mv(reg, format!("a{}", index).as_str());
                }
                else {
                    f.lw(reg, "sp", (spoff + (index - 8) * 4) as i32); // load args from stack
                
                }
            },
            _ => unreachable!()
        };
        f.update_temp_reg(res);
        Ok(())
    }

    pub fn reload_value_from_reg(&self, f: &'file mut Writer, reg: &str, temp_reg: &'static str) -> Result<()> {
        let res = f.reg_temp;
        f.update_temp_reg(temp_reg);
        match self{


            Self::Global(name) => {
                f.la(temp_reg, name);
                f.sw(reg, temp_reg, 0);
            },
            Self::LocalVar(slot) => {
                assert!(slot.stack.is_some());
                f.sw(reg, "sp", slot.stackslot_offset().unwrap() as i32);
            }
            Self::FuncArg(index) => {
                if *index < 8 {
                    f.mv(format!("a{}", index).as_str(), reg);
                }
                else {
                    f.sw(reg, "sp", ((index - 8)*4) as i32); // write args to stack
                }
            }
            Self::Void => {}
            _ => unreachable!()
        };
        f.update_temp_reg(res);
        Ok(())
    }
}

