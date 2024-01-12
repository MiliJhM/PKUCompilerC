/*
    Code Generator:
        Compile KoopaIR AST to RISC-V Assembly Code 
*/
use super::asm_generator::*;
use super::program_manager::*;
use super::asm_value::*;
use super::reg_manager::*;

use koopa::ir::entities::*;
use koopa::ir::*;

use std::fs::File;
use std::io::{Write, Result};

// * trait AsmGenerator - 递归生成汇编代码
pub trait AsmGenerator<'prog, 'file> {
    type Out;

    fn generate(&mut self, program: &mut ProgramManager<'prog>, f: &mut Writer<'file>) -> Result<Self::Out>;
}

// * trait AsmValueGenerator - 上一trait的扩展，用于对IR设计的value各个类型传递其Data
pub trait AsmValueGenerator<'prog, 'file> {
    type Out;

    fn generate(&mut self, program: &mut ProgramManager<'prog>, f: &mut Writer<'file>,  value: &ValueData) -> Result<Self::Out>;
}

impl<'prog, 'file> AsmGenerator<'prog, 'file> for Program{
    type Out = ();
    
    fn generate(&mut self, program: &mut ProgramManager<'prog>, f: &mut Writer<'file>) -> Result<Self::Out>{

        let file_m = f.file_mut();

        // .data section
        writeln!(file_m, "  .data")?;
        for &global in self.inst_layout(){
            let valdata = self.borrow_value(global);
            let valname = valdata.name().as_ref().unwrap()[1..].to_string();

            program.insert_value(global, valname);
            
            writeln!(file_m, "  .globl {}", valname)?;
            writeln!(file_m, "{}:", valname)?;
            valdata.generate(program, f)?;
            writeln!(file_m)?;
        }

        // .text section
        writeln!(file_m, "  .text")?;
        for &func in self.func_layout(){
            program.set_cur_func(FunctionInterface::new(func));
            self.func(func).generate(program, f)?;
        }

        return Ok(());
    }
}

impl<'prog, 'file> AsmGenerator<'prog, 'file> for FunctionData {
    type Out = ();

    fn generate(&mut self, program: &mut ProgramManager<'prog>, f: &mut Writer<'file>) -> Result<Self::Out> {
        if self.layout().entry_bb().is_none() {
            return Ok(());
        }


        let func_interface = program.cur_func_mut().unwrap();
        for value in self.dfg().values().values(){
            if value.kind().is_local_inst() && !value.used_by().is_empty(){
                func_interface.alloc_new_slot(value);
            }

            else if let ValueKind::Call(val) = value.kind(){
                func_interface.update_max_arg_num(call.args().len());
            }
        }

        writeln!(f.file_mut(), "  # Function {}, arg_num: {}", &self.name().to_string()[1..], func_interface.get_arg_num().unwrap_or_default())?;

        for (bb, bb_data) in self.dfg().bbs() {
            func_interface.set_bb_name(*bb, bb_data.name());
        }

        f.func_entry(self.name(), func_interface);

        for (bb, bb_node) in self.layout().bbs() {
            let bb_name = func_interface.get_bb_name(*bb);
            writeln!(f.file_mut(), "{}:", bb_name)?;
            for (&val_handle, _) in bb_node.insts() {
                self.dfg().value(val_handle).generate(program, f)?;
            }

        }
        writeln!(f.file_mut());

        return Ok(());
    }
}

impl<'prog, 'file> AsmGenerator<'prog, 'file> for ValueData {
    type Out = AsmValue;

    fn generate(&mut self, program: &mut ProgramManager<'prog>, f: &mut Writer<'file>) -> Result<Self::Out> {
        

        
    }
}