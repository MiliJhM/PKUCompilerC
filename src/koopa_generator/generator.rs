use std::collections::HashMap;

use std::io::Write;
use crate::ast::ast_def::*;
use super::function_interface;
use super::namespace::*;
use super::CResult;
use super::function_interface::*;
use koopa::ir::values::FuncArgRef;
use koopa::ir::{*, builder_traits::*};


impl FunctionInterface{
    pub fn alloc_new_value(&mut self, program: &mut Program, typ: Type, name: Option<&str>) -> Value{
        let alloc = self.value_builder(program).alloc(typ);
        if let Some(name) = name{
            self.get_dfg_mut(program).set_value_name(alloc, Some(format!("@{}",name)));
        }
        self.push_inst_to_bb(program, self.get_bblock_from_list("%entry"), alloc);
        alloc
    }
/* 
    pub fn entry_finish(&self, program: &mut Program, next: BasicBlock){
        let jump_inst =self.value_builder(program).jump(next);
        self.get_bblock_from_name(program, "%entry").insts_mut().push_key_back(jump_inst);
    }
*/
    pub fn func_finish(&mut self, program: &mut Program) {
        let entry_block = self.get_bblock_from_list("%entry");
        let end_block = self.get_bblock_from_list("%end");
        let func_block = self.get_bblock_from_list("%func");

        let jump_inst =self.value_builder(program).jump(func_block);
        self.push_inst_to_bb(program, entry_block, jump_inst);
        let jump_inst = self.value_builder(program).jump(end_block);
        self.push_inst_to_bb(program, func_block, jump_inst);
        self.push_bblock(program, end_block);
        let return_value_load_to_0 = None;
        if self.get_ret().is_some() {
            let return_value_load_to_0 = self.value_builder(program).load(self.get_ret().clone().unwrap());
            self.push_inst_to_bb(program, end_block, return_value_load_to_0);
        }
        let ret = self.value_builder(program).ret(return_value_load_to_0); // inst“ret” need param?
        self.push_inst_to_bb(program, end_block, ret);
    }
}
pub trait GenerateKoopa {
    type Out;
    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out>;
}

impl CompileInit{
    fn load_lib_func(&self, namespace: &mut Namesp, program: &mut Program){
        let mut create_lib_func = |name, params_type, return_type| {
            let new_func = program.new_func(FunctionData::new_decl( format!("@{}", name), params_type, return_type));
            namespace.new_func(name, new_func).unwrap();
        };
        create_lib_func("getint", Vec::new(), Type::get_i32());
        create_lib_func("getch", Vec::new(), Type::get_i32());
        create_lib_func("getarray", vec![Type::get_pointer(Type::get_i32())], Type::get_i32());
        create_lib_func("putint", vec![Type::get_i32()], Type::get_unit());
        create_lib_func("putch", vec![Type::get_i32()], Type::get_unit());
        create_lib_func("putarray", vec![Type::get_i32(), Type::get_pointer(Type::get_i32())], Type::get_unit());
        create_lib_func("starttime", Vec::new(), Type::get_unit());
        create_lib_func("stoptime", Vec::new(), Type::get_unit());
    }
}

impl GenerateKoopa for CompileInit {
    type Out = ();
    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {
        self.load_lib_func(namespace, program);
        
        for decl_or_func in &self.init {
            decl_or_func.generate(namespace, program)?;
        }
        return Ok(());
    }
}

impl GenerateKoopa for DeclOrFunc {
    type Out = ();
    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {
        match self {
            DeclOrFunc::Decl(decl) => {
                decl.generate(namespace, program)?;
            }
            DeclOrFunc::Func(func) => {
                func.generate(namespace, program)?;
            }
        }
        return Ok(());
    }
}

// TODO: Finish this:)
impl GenerateKoopa for Decl {
    type Out = ();
    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {
        return Ok(());
    }
}

impl GenerateKoopa for FuncDef {
    type Out = ();
    fn generate<'a>(&self, namespace: & mut Namesp, program: & mut Program) -> CResult<Self::Out> {
        let ret_type = match self.func_type {
            FuncType::Void => Type::get_unit(),
            FuncType::Int => Type::get_i32(),
        };
        let args_type = self.func_params.iter().map(|param| param.generate(namespace, program)).collect::<CResult<Vec<Type>>>()?;
        let mut func_data: FunctionData = FunctionData::new("@".to_owned()+self.func_name.as_str(), args_type, ret_type);
        let mut new_func = program.new_func(func_data);
        

        let mut return_val = None;
        if matches!(self.func_type, FuncType::Int) {
            let new_alloc = program.func_mut(new_func).dfg_mut().new_value().alloc(Type::get_i32());
            program.func_mut(new_func).dfg_mut().set_value_name(new_alloc, Some("%ret".to_string()));
            return_val = Some(new_alloc);
            // namespace.set_ret_value(return_val);
        }
        let mut func_interface = FunctionInterface::new(new_func, return_val);

        // TODO: define Blocks
        let entry_block = func_interface.new_bblock(program, "%entry");
        let func_block = func_interface.new_bblock(program, "%func");
        let end_block = func_interface.new_bblock(program, "%end");

        //namespace.set_ret_value(return_val);

        let params = func_interface.get_func_data(program).params().to_owned();
        // Constructing the function using basic blocks
        func_interface.push_bblock(program, entry_block);
        // Alloc the return value
        if return_val.is_some() {
            func_interface.push_inst_to_bb(program, entry_block, return_val.unwrap());
        }
        func_interface.push_bblock(program, func_block);
        namespace.enter_new_scope();
        for(func_param, param_value) in self.func_params.iter().zip(params){
            let ty = program.func(new_func).dfg().value(param_value).ty().clone();
            let alloc = func_interface.alloc_new_value(program, ty, Some("pa"));
            let new_store = func_interface.value_builder(program).store(param_value, alloc);
            func_interface.push_inst_to_bb(program, func_block, new_store);
            namespace.new_value(func_param.param_id.as_str(), NamespValue::Var(alloc), false)?;
        }
        // dump
        namespace.new_func(&self.func_name, new_func)?;
        namespace.cur_function = Some(func_interface);

        self.func_body.generate(namespace, program)?;
        
        // end
        let mut func_interface = namespace.cur_function.take().unwrap(); // Take owned before exit scope
        namespace.exit_now_scope();

        func_interface.func_finish(program);
        return Ok(());
    }
}

impl GenerateKoopa for Param {
    type Out = Type;
    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {
        Ok(
            match &self.param_dims{
            None => Type::get_i32(),
            Some(dims) => Type::get_i32(), // TODO:: Array Pointer
            }
        )
    }
}

impl GenerateKoopa for Block {
    type Out = ();
    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {
        namespace.enter_new_scope();
        for block_item in &self.items {
            block_item.generate(namespace, program)?;
        }
        namespace.exit_now_scope();
        return Ok(());
    }
}

impl GenerateKoopa for BlockItem{
    type Out = ();
    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {
        match self {
            BlockItem::Decl(decl) => {
                decl.generate(namespace, program)?;
            }
            BlockItem::Stmt(stmt) => {
                stmt.generate(namespace, program)?;
            }
        }
        return Ok(());
    }
}

impl GenerateKoopa for Stmt{
    type Out = ();
    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {
        match self {
            Stmt::ReturnStmt(ret_stmt) => {
                ret_stmt.generate(namespace, program)?;
            }
            Stmt::AssignStmt(assign_stmt) => {
                assign_stmt.generate(namespace, program)?;
            }
            Stmt::ExprStmt(expr_stmt) => {
                expr_stmt.generate(namespace, program)?;
            }
            Stmt::BlockStmt(block_stmt) => {
                block_stmt.generate(namespace, program)?;
            }
            Stmt::IfStmt(if_stmt) => {
                if_stmt.generate(namespace, program)?;
            }
            Stmt::WhileStmt(while_stmt) => {
                while_stmt.generate(namespace, program)?;
            }
            Stmt::BreakStmt(break_stmt) => {
                break_stmt.generate(namespace, program)?;
            }
            Stmt::ContinueStmt(continue_stmt) => {
                continue_stmt.generate(namespace, program)?;
            }
        }
        return Ok(());
    }
}

impl GenerateKoopa for AssignStmt {
    type Out = ();
    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {

        return Ok(());
    }
}

impl GenerateKoopa for ExprStmt {
    type Out = ();
    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {

        return Ok(());
    }
}

impl GenerateKoopa for ReturnStmt {
    type Out = ();
    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {

        return Ok(());
    }
}

impl GenerateKoopa for IfStmt {
    type Out = ();
    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {

        return Ok(());
    }
}

impl GenerateKoopa for WhileStmt {
    type Out = ();
    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {

        return Ok(());
    }
}

impl GenerateKoopa for BreakStmt {
    type Out = ();
    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {
        return Ok(());
    }
}

impl GenerateKoopa for ContinueStmt {
    type Out = ();
    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {
        return Ok(());
    }
}

