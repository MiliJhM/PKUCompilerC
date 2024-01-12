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
use koopa::ir::ValueKind;
use koopa::ir::values::*;
use std::fs::File;
use std::io::{Write, Result};

// * trait AsmGenerator - 递归生成汇编代码
pub trait AsmGenerator<'prog, 'file> {
    type Out;

    fn generate(&self, program: &mut ProgramManager<'prog>, f: &mut Writer<'file>) -> Result<Self::Out>;
}

// * trait AsmValueGenerator - 上一trait的扩展，用于对IR设计的value各个类型传递其Data
pub trait AsmValueGenerator<'prog, 'file> {
    type Out;

    fn generate(&self, program: &mut ProgramManager<'prog>, f: &mut Writer<'file>,  value: &ValueData) -> Result<Self::Out>;
}

impl<'prog, 'file> AsmGenerator<'prog, 'file> for Program{
    type Out = ();
    
    fn generate(&self, program: &mut ProgramManager<'prog>, f: &mut Writer<'file>) -> Result<Self::Out>{

        if !self.inst_layout().is_empty()
        {
            let file_m = f.file_mut();
            // .data section
            writeln!(file_m, "  .data")?;
        }
        for &global in self.inst_layout(){
            let valdata = self.borrow_value(global);
            let valname = &valdata.name().as_ref().unwrap()[1..];

            program.insert_value(global, valname.to_string());
            {
                let file_m = f.file_mut();
                writeln!(file_m, "  .globl {}", valname)?;
                writeln!(file_m, "{}:", valname)?;
            }
            valdata.clone().generate(program, f)?;
            {
                let file_m = f.file_mut();
                writeln!(file_m)?;
            }

        }

        {
            let file_m = f.file_mut();
            // .data section
            writeln!(file_m, "  .text")?;
        }
        for &func in self.func_layout(){
            program.set_cur_func(FunctionInterface::new(func));
            self.func(func).clone().generate(program, f)?;
        }

        return Ok(());
    }
}

impl<'prog, 'file> AsmGenerator<'prog, 'file> for FunctionData {
    type Out = ();

    fn generate(&self, program: &mut ProgramManager<'prog>, f: &mut Writer<'file>) -> Result<Self::Out> {
        if self.layout().entry_bb().is_none() {
            return Ok(());
        }


        let func_interface = program.cur_func_mut().unwrap();
        for value in self.dfg().values().values(){
            if value.kind().is_local_inst() && !value.used_by().is_empty(){
                func_interface.alloc_new_slot(value);
            }
            if let ValueKind::Call(val) = value.kind(){
                func_interface.update_max_arg_num(val.args().len());
            }
        }

        writeln!(f.file_mut(), "  # Function {}, arg_num: {}", &self.name().to_string()[1..], func_interface.get_arg_num().unwrap_or_default())?;

        for (bb, bb_data) in self.dfg().bbs() {
            func_interface.set_bb_name(*bb, bb_data.name());
        }

        f.func_entry(self.name(), func_interface);

        let func_interface = program.cur_func().unwrap();

        for (bb, bb_node) in self.layout().bbs() {
            let bb_name = bb.generate(program, f)?;

            writeln!(f.file_mut(), "{}:", &bb_name.as_str())?;
            for (&val_handle, _) in bb_node.insts() {
                self.dfg().value(val_handle).generate(program, f)?;
            }

        }
        writeln!(f.file_mut());

        return Ok(());
    }
}

impl<'prog, 'file> AsmGenerator<'prog, 'file> for BasicBlock{
    type Out = String;
    fn generate(&self, program: &mut ProgramManager<'prog>, f: &mut Writer<'file>) -> Result<Self::Out> {
        Ok(program.cur_func().unwrap().get_bb_name(*self).to_string())
    }
}

impl<'prog, 'file> AsmGenerator<'prog, 'file> for Value{
    type Out = AsmValue;

    fn generate(&self, program: &mut ProgramManager<'prog>, f: &mut Writer<'file>) -> Result<Self::Out> {
        if self.is_global() {
            Ok(AsmValue::Global(program.value_name(*self).clone()))
        }
        else {
            let func_interface = program.cur_func().unwrap();
            let val_data = program.program().func(func_interface.get_func()).dfg().value(*self);
            let ret = match val_data.kind() {
                ValueKind::Integer(v) => AsmValue::Const(v.value()),
                ValueKind::FuncArgRef(v) => AsmValue::FuncArg(v.index()),
                _ => {
                    let new_slot = func_interface.stack_offset_resize(val_data);
                    match new_slot {
                        Some(slot) => AsmValue::LocalVar(slot),
                        None => AsmValue::Void,
                    }
                }
            };
            Ok(ret)
        }
    }
}

impl<'prog, 'file> AsmGenerator<'prog, 'file> for ValueData {
    type Out = ();

    fn generate(&self, program: &mut ProgramManager<'prog>, f: &mut Writer<'file>) -> Result<Self::Out> {
        
        match self.kind() {
            ValueKind::ZeroInit(v) => v.generate(program, f, self),
            ValueKind::Load(v) => v.generate(program, f, self),
            ValueKind::GetPtr(v) => v.generate(program, f, self),
            ValueKind::GetElemPtr(v) => v.generate(program, f, self),
            ValueKind::Binary(v) => v.generate(program, f, self),
            ValueKind::Call(v) => v.generate(program, f, self),
            ValueKind::Integer(v) => v.generate(program, f),
            ValueKind::Aggregate(v) => v.generate(program, f),
            ValueKind::GlobalAlloc(v) => v.generate(program, f),
            ValueKind::Store(v) => v.generate(program, f),
            ValueKind::Branch(v) => v.generate(program, f),
            ValueKind::Jump(v) => v.generate(program, f),
            ValueKind::Return(v) => v.generate(program, f),
            _ => Ok(())
        }
    }
}

impl<'prog, 'file> AsmGenerator<'prog, 'file> for Integer {
    type Out = ();

    fn generate(&self, program: &mut ProgramManager<'prog>, f: &mut Writer<'file>) -> Result<Self::Out> {
        writeln!(f.file_mut(), "  .word {}", self.value())
    }
}

impl<'prog, 'file> AsmGenerator<'prog, 'file> for Aggregate {
    type Out = ();

    fn generate(&self, program: &mut ProgramManager<'prog>, f: &mut Writer<'file>) -> Result<Self::Out> {
        for &val in self.elems(){
            program.program().borrow_value(val).generate(program, f)?;
        }
        Ok(())
    }
}

impl<'prog, 'file> AsmGenerator<'prog, 'file> for GlobalAlloc {
    type Out = ();

    fn generate(&self, program: &mut ProgramManager<'prog>, f: &mut Writer<'file>) -> Result<Self::Out> {
        program.program().borrow_value(self.init()).generate(program, f)?;
        Ok(())
    }
}

impl<'prog, 'file> AsmGenerator<'prog, 'file> for Store {
    type Out = ();

    fn generate(&self, program: &mut ProgramManager<'prog>, f: &mut Writer<'file>) -> Result<Self::Out> {
        let spoff = program.cur_func().unwrap().sp_offset();
        let val = self.value().generate(program, f)?;
        match val {
            AsmValue::FuncArg(v) => val.arg_to_reg(f, "t0", spoff)?,
            _ => val.normal_to_reg(f, "t0")?,
        }
        let dst = self.dest().generate(program, f)?;
        if dst.is_ptr() {
            dst.normal_to_reg(f, "t1")?;
            f.update_temp_reg("t2");
            f.sw("t0", "t1", 0);
            f.update_temp_reg("t0");
        }
        else {
            dst.reload_value_from_reg(f, "t0", "t1");
        }
        Ok(())
    }
}

impl<'prog, 'file> AsmGenerator<'prog, 'file> for Branch {
    type Out = ();

    fn generate(&self, program: &mut ProgramManager<'prog>, f: &mut Writer<'file>) -> Result<Self::Out> {
        self.cond().generate(program, f)?.normal_to_reg(f, "t0");
        let func_interface = program.cur_func().unwrap();
        let tto_name = func_interface.get_bb_name(self.true_bb());
        let fto_name = func_interface.get_bb_name(self.false_bb());
        f.bnez("t0", &tto_name);
        f.j(&fto_name);
        Ok(())
    }
}

impl<'prog, 'file> AsmGenerator<'prog, 'file> for Jump {
    type Out = ();

    fn generate(&self, program: &mut ProgramManager<'prog>, f: &mut Writer<'file>) -> Result<Self::Out>{
        let func_interface = program.cur_func().unwrap();
        let to_name = func_interface.get_bb_name(self.target());
        f.j(&to_name);
        Ok(())
    }
}

impl<'prog, 'file> AsmGenerator<'prog, 'file> for Return {
    type Out = ();

    fn generate(&self, program: &mut ProgramManager<'prog>, f: &mut Writer<'file>) -> Result<Self::Out> {
        if let Some(val) = self.value() {
            val.clone().generate(program, f)?.normal_to_reg(f, "a0");
        }
        f.func_end(program.cur_func().unwrap());
        Ok(())
    }
}

impl<'prog, 'file> AsmValueGenerator<'prog, 'file> for ZeroInit {
    type Out = ();

    fn generate(&self, program: &mut ProgramManager<'prog>, f: &mut Writer<'file>, val: &ValueData) -> Result<Self::Out> {
        let valname = val.name();
        writeln!(f.file_mut(), "  .zero {}", val.ty().size())
    }
}

impl<'prog, 'file> AsmValueGenerator<'prog, 'file> for Load {
    type Out = ();

    fn generate(&self, program: &mut ProgramManager<'prog>, f: &mut Writer<'file>,  value: &ValueData) -> Result<Self::Out> {
        let src = self.src().generate(program, f)?;
        src.normal_to_reg(f, "t0")?;
        if src.is_ptr(){
            f.update_temp_reg("t1");
            f.lw("t0", "t0", 0)?;
            f.update_temp_reg("t0");
        }
        AsmValue::LocalVar(program.cur_func().unwrap().stack_offset_resize(value).unwrap()).reload_value_from_reg(f, "t0", "t1")
    }
}

impl<'prog, 'file> AsmValueGenerator<'prog, 'file> for GetPtr {
    type Out = ();

    fn generate(&self, program: &mut ProgramManager<'prog>, f: &mut Writer<'file>,  value: &ValueData) -> Result<Self::Out> {
        let src = self.src().generate(program, f)?;
        if src.is_ptr(){
            src.normal_to_reg(f, "t0");
        }
        else {
            src.load_addr_to_reg(f, "t0");
        }
        self.index().generate(program, f)?.normal_to_reg(f, "t1")?;
        let size = match value.ty().kind() {
            TypeKind::Pointer(b) => b.size(),
            _ => unreachable!()
        };
        f.update_temp_reg("t2");
        f.muli("t1", "t1", size as i32);
        f.op2("add", "t0", "t0", "t1");
        f.update_temp_reg("t0");

        match program.cur_func().unwrap().stack_offset_resize(value) {
            Some(v) => AsmValue::LocalVar(v).reload_value_from_reg(f, "t0", "t1"),
            None => AsmValue::Void.reload_value_from_reg(f, "t0", "t1")
        };
        Ok(())
    }
}

impl<'prog, 'file> AsmValueGenerator<'prog, 'file> for GetElemPtr {
    type Out = ();

    fn generate(&self, program: &mut ProgramManager<'prog>, f: &mut Writer<'file>,  value: &ValueData) -> Result<Self::Out> {
        let src = self.src().generate(program, f)?;
        if src.is_ptr(){
            src.normal_to_reg(f, "t0");
        }
        else {
            src.load_addr_to_reg(f, "t0");
        }
        self.index().generate(program, f)?.normal_to_reg(f, "t1")?;
        let size = match value.ty().kind() {
            TypeKind::Pointer(b) => b.size(),
            _ => unreachable!()
        };
        f.update_temp_reg("t2");
        f.muli("t1", "t1", size as i32);
        f.op2("add", "t0", "t0", "t1");
        f.update_temp_reg("t0");

        match program.cur_func().unwrap().stack_offset_resize(value) {
            Some(v) => AsmValue::LocalVar(v).reload_value_from_reg(f, "t0", "t1"),
            None => AsmValue::Void.reload_value_from_reg(f, "t0", "t1")
        };
        Ok(())
    }
}

impl<'prog, 'file> AsmValueGenerator<'prog, 'file> for Binary {
    type Out = ();

    fn generate(&self, program: &mut ProgramManager<'prog>, f: &mut Writer<'file>,  value: &ValueData) -> Result<Self::Out> {
        self.lhs().generate(program, f)?.normal_to_reg(f, "t0")?;
        self.rhs().generate(program, f)?.normal_to_reg(f, "t1")?;
        f.update_temp_reg("t2");
        match self.op() {
            BinaryOp::Add => f.op2("add", "t0", "t0", "t1")?,
            BinaryOp::Sub => f.op2("sub", "t0", "t0", "t1")?,
            BinaryOp::Mul => f.op2("mul", "t0", "t0", "t1")?,
            BinaryOp::Div => f.op2("div", "t0", "t0", "t1")?,
            BinaryOp::Mod => f.op2("rem", "t0", "t0", "t1")?,
            BinaryOp::And => f.op2("and", "t0", "t0", "t1")?,
            BinaryOp::Or => f.op2("or", "t0", "t0", "t1")?,
            BinaryOp::Xor => f.op2("xor", "t0", "t0", "t1")?,
            BinaryOp::Shl => f.op2("sll", "t0", "t0", "t1")?,
            BinaryOp::Shr => f.op2("srl", "t0", "t0", "t1")?,
            BinaryOp::Sar => f.op2("sra", "t0", "t0", "t1")?,

            BinaryOp::NotEq => {
                f.op2("xor", "t0", "t0", "t1")?;
                f.op1("snez", "t0", "t0")?;
            },
            BinaryOp::Eq => {
                f.op2("xor", "t0", "t0", "t1")?;
                f.op1("seqz", "t0", "t0")?;
            },
            BinaryOp::Gt => f.op2("sgt", "t0", "t0", "t1")?,
            BinaryOp::Lt => f.op2("slt", "t0", "t0", "t1")?,
            BinaryOp::Ge => {
                f.op2("slt", "t0", "t0", "t1")?;
                f.op1("seqz", "t0", "t0")?;
            },
            BinaryOp::Le => {
                f.op2("sgt", "t0", "t0", "t1")?;
                f.op1("seqz", "t0", "t0")?;
            },
        }
        f.update_temp_reg("t0");
        match program.cur_func().unwrap().stack_offset_resize(value) {
            Some(v) => AsmValue::LocalVar(v).reload_value_from_reg(f, "t0", "t1"),
            None => AsmValue::Void.reload_value_from_reg(f, "t0", "t1")
        };
        Ok(())
    }
}

impl<'prog, 'file> AsmValueGenerator<'prog, 'file> for Call {
    type Out = ();

    fn generate(&self, program: &mut ProgramManager<'prog>, f: &mut Writer<'file>,  value: &ValueData) -> Result<Self::Out> {
        let mut arglist = Vec::new();
        for arg in self.args() {
            arglist.push(arg.clone().generate(program, f)?);
        }

        for (i, arg) in arglist.iter().enumerate() {
            arg.normal_to_reg(f, "t0");
            AsmValue::FuncArg(i).reload_value_from_reg(f, "t0", "t1");
        }

        let callee_name = &program.program().func(self.callee()).name()[1..];
        f.call(callee_name);
        if !value.used_by().is_empty() {
            match program.cur_func().unwrap().stack_offset_resize(value) {
                Some(v) => AsmValue::LocalVar(v).reload_value_from_reg(f, "a0", "t1"),
                None => AsmValue::Void.reload_value_from_reg(f, "a0", "t1")
            };
            Ok(())
        }
        else {
            Ok(())
        }
    }
}