use std::collections::HashMap;

use std::io::Write;
use crate::ast::ast_def::*;
use super::CompileError;
use super::function_interface;
use super::namespace::*;
use super::CResult;
use super::function_interface::*;
use koopa::ir;
use koopa::ir::values::FuncArgRef;
use koopa::ir::values::GetElemPtr;
use koopa::ir::{*, builder_traits::*};
use super::const_evaluator::*;



fn get_type(namespace: &mut Namesp, program: &mut Program, val: Value) -> Type {
    if val.is_global() {
        return program.borrow_value(val).ty().clone();
    }
    else {
        return namespace.get_cur_func_interf().unwrap()
            .get_dfg_mut(program)
            .value(val)
            .ty()
            .clone()
        ;
    }
}

impl FunctionInterface{
    pub fn alloc_new_value(&mut self, program: &mut Program, typ: Type, name: Option<&str>) -> Value{
        let alloc = self.value_builder(program).alloc(typ);
        if let Some(name) = name{
            self.get_dfg_mut(program).set_value_name(alloc, Some(format!("@{}",name)));
        }
        self.push_inst_to_bb(program, self.get_bblock_from_list("%entry"), alloc);
        return alloc;
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
        self.push_inst_to_bb(program, self.current_bb(), jump_inst);
        self.push_bblock(program, end_block);
        let mut return_value_load_to_0 = None;
        if self.get_ret().is_some() {
            return_value_load_to_0 = Some(self.value_builder(program).load(self.get_ret().clone().unwrap()));
            self.push_inst_to_bb(program, end_block, return_value_load_to_0.unwrap());
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
        match self {
            Self::Const(const_decl) => {
                const_decl.generate(namespace, program)
            },
            Self::Var(var_decl) => {
                var_decl.generate(namespace, program)
            },
        }
    }
}

impl GenerateKoopa for ConstDecl {
    type Out = ();
    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {
        for const_def in &self.defs {
            const_def.generate(namespace, program)?;
        }
        return Ok(());
    }
}

fn dim_vec_to_type(dims: &Vec<ConstExpr>, namespace: &mut Namesp) -> CResult<Type> {
    dims.iter().rev()
    .fold(Ok(Type::get_i32()), |acc, dim| {
        let len = dim.const_eval(namespace).unwrap();
        if len>=1 {
            Ok(Type::get_array(acc?, len as usize))
        }
        else{
            return Err(CompileError::InvalidInit("".to_owned()));
        }
    })

}

impl GenerateKoopa for ConstDef {
    type Out = ();

    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {
        let ty_from_dims = dim_vec_to_type(&self.dims, namespace);
        let ty_from_dims = match ty_from_dims {
            Ok(ty) => ty,
            Err(_) => {
                let ty = Type::get_i32();
                if namespace.is_global() {
                    let init_data = program.new_value().zero_init(ty);
                    let value = program.new_value().global_alloc(init_data);
                    program.set_value_name(value, Some(format!("@{}", self.id)));
                    return Ok(());
                }
                else {
                    let func_interface = namespace.get_cur_func_interf_mut()?;
                    let alloc = func_interface.alloc_new_value(program, ty, Some(&self.id));
                    let value = func_interface.value_builder(program).integer(0);
                    let value = func_interface.value_builder(program).store(value, alloc);
                    return Ok(());
                };
            },
        };

        let init = self.init_val.generate(namespace, program)?.init_rebuild(&ty_from_dims)?;

        if ty_from_dims.is_i32() {
            match init {
                InitValue::Const(val) => {
                    namespace.new_value(&self.id, NamespValue::ConstInt(val), true);
                },
                _ => unreachable!(),
            }
        }
        else {
            let value = if namespace.is_global() {
                let init_data = init.into_const(program, namespace)?;
                let value = program.new_value().global_alloc(init_data);
                program.set_value_name(value, Some(format!("@{}", self.id)));
                value
            }
            else {
                let func_interface = namespace.get_cur_func_interf_mut()?;
                let alloc = func_interface.alloc_new_value(program, ty_from_dims, Some(&self.id));
                init.into_ptr_stored(program, namespace, alloc);
                alloc
            };
            namespace.new_value(&self.id, NamespValue::Var(value), true);
        }

        return Ok(());
    }
}

impl GenerateKoopa for ConstInitVal {
    type Out = InitValue;

    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {
        let result = match self{
            Self::Expr(expr) => InitValue::Const(expr.const_eval(namespace).unwrap()),

            Self::List(list) => {
                let mut result = Vec::new();
                for init_val in list {
                    result.push(init_val.generate(namespace, program)?);
                }
                InitValue::List(result)
            },
        };
        return Ok(result);
    }
}


impl GenerateKoopa for VarDecl {
    type Out = ();

    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {
        for var_def in &self.defs {
            var_def.generate(namespace, program)?;
        }
        return Ok(());
    }
}

impl GenerateKoopa for VarDef {
    type Out = ();

    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {
        let type_from_dims = dim_vec_to_type(&self.dims, namespace);
        let type_from_dim = match type_from_dims {
            Ok(ty) => ty,
            Err(_) => {
                let ty = Type::get_i32();
                if namespace.is_global() {
                    let init_data = program.new_value().zero_init(ty);
                    let value = program.new_value().global_alloc(init_data);
                    program.set_value_name(value, Some(format!("@{}", self.id)));
                    return Ok(());
                }
                else {
                    let func_interface = namespace.get_cur_func_interf_mut()?;
                    let alloc = func_interface.alloc_new_value(program, ty, Some(&self.id));
                    let value = func_interface.value_builder(program).integer(0);
                    let value = func_interface.value_builder(program).store(value, alloc);
                    return Ok(());
                };
            },
        };
        let init = match &self.init_val {
            Some(init) => Some(init.generate(namespace, program)?.init_rebuild(&type_from_dim)?),
            None => None,
        };
        let value = if namespace.is_global() {
            let init_data = match init {
                Some(init) => init.into_const(program, namespace)?,
                None => program.new_value().zero_init(type_from_dim)
            };
            let value = program.new_value().global_alloc(init_data);
            program.set_value_name(value, Some(format!("@{}", self.id)));
            value
        }
        else {
            let func_interface = namespace.get_cur_func_interf_mut()?;
            let alloc = func_interface.alloc_new_value(program, type_from_dim, Some(&self.id));
            if let Some(init) = init {
                init.into_ptr_stored(program, namespace, alloc);
            }
            alloc
        };
        namespace.new_value(&self.id, NamespValue::Var(value), false);
        return Ok(());
    }
}

impl GenerateKoopa for InitVal {
    type Out = InitValue;

    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {
        let result = match self{
            Self::Expr(expr) =>{
                if namespace.is_global() {
                    InitValue::Const(expr.const_eval(namespace).ok_or(CompileError::InvalidInit("".to_string()))?)
                }
                else {
                    InitValue::Var(expr.generate(namespace, program)?.into_value(program, namespace)?)
                }
            },

            Self::List(list) => {
                let mut result = Vec::new();
                for init_val in list {
                    result.push(init_val.generate(namespace, program)?);
                }
                InitValue::List(result)
            },
        };
        return Ok(result);
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
        // Take owned before exit scope
        
        let mut func_interface = namespace.get_cur_func_interf_mut()?;
        func_interface.func_finish(program);
        namespace.exit_now_scope();
        return Ok(());
    }
}

impl GenerateKoopa for Param {
    type Out = Type;
    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {
        Ok(
            match &self.param_dims{
                None => Type::get_i32(),
                Some(dims) => Type::get_pointer(dim_vec_to_type(dims, namespace)?),
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
        let exprvalue = self.expr.generate(namespace, program)?.into_value(program, namespace)?;
        let lval = self.lval.generate(namespace, program)?.into_lvptr()?;
        let function_interface = namespace.get_cur_func_interf()?;
        let store = function_interface.value_builder(program).store(exprvalue, lval);
        function_interface.push_inst_to_bb(program, function_interface.current_bb(), store);
        return Ok(());
    }
}

impl GenerateKoopa for ExprStmt {
    type Out = ();
    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {

        if let Some(expr) = &self.expr {
            expr.generate(namespace, program)?;
        }

        return Ok(());
    }
}

impl GenerateKoopa for ReturnStmt {
    type Out = ();
    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {
        if let Some(ret) = namespace.get_cur_func_interf()?.get_ret() {
            if let Some(ret_exp) = &self.expr {
                let ret_exp = ret_exp.generate(namespace, program)?.into_value(program, namespace)?;
                let func_interface = namespace.get_cur_func_interf()?;
                let store = func_interface.value_builder(program).store(ret_exp, ret);
                func_interface.push_inst_to_bb(program, func_interface.current_bb(), store);
            }
            else if self.expr.is_some() {
                return Err(CompileError::InvalidReturn("".to_owned()));
            }
        }
        let func_interface = namespace.get_cur_func_interf_mut()?;
        let jump = func_interface.value_builder(program).jump(func_interface.get_bblock_from_list("%end"));
        func_interface.push_inst_to_bb(program, func_interface.current_bb(), jump);
        let new_next = func_interface.new_anomynous_bblock(program);
        func_interface.push_bblock(program, new_next);
        return Ok(());
    }
}

impl GenerateKoopa for IfStmt {
    type Out = ();
    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {
        let cond_val = self.condition.generate(namespace, program)?.into_value(program, namespace)?;
        
        let func_interface = namespace.get_cur_func_interf_mut()?;
        let then_block = func_interface.new_bblock(program, "%if_then");
        let else_block = func_interface.new_bblock(program, "%if_else");
        let end_block = func_interface.new_bblock(program, "%if_end");
        let branch = func_interface.value_builder(program).branch(cond_val, then_block, else_block);
        func_interface.push_inst_to_bb(program, func_interface.current_bb(), branch);

        func_interface.push_bblock(program, then_block);
        self.then_stmt.generate(namespace, program)?;
        let func_interface = namespace.get_cur_func_interf_mut()?;
        let jump = func_interface.value_builder(program).jump(end_block);
        func_interface.push_inst_to_bb(program, func_interface.current_bb(), jump);

        func_interface.push_bblock(program, else_block);
        if let Some(else_stmt) = &self.else_stmt {
            else_stmt.generate(namespace, program)?;
        }

        let func_interface = namespace.get_cur_func_interf_mut()?;
        let jump = func_interface.value_builder(program).jump(end_block);
        func_interface.push_inst_to_bb(program, func_interface.current_bb(), jump);

        func_interface.push_bblock(program, end_block);
        return Ok(());
    }
}

impl GenerateKoopa for WhileStmt {
    type Out = ();
    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {
        let func_interface = namespace.get_cur_func_interf_mut()?;
        let while_entry = func_interface.new_bblock(program, "%while_entry");

        let jump_into_while = func_interface.value_builder(program).jump(while_entry);
        func_interface.push_inst_to_bb(program, func_interface.current_bb(), jump_into_while);
        func_interface.push_bblock(program, while_entry);

        let cond_val = self.condition.generate(namespace, program)?.into_value(program, namespace)?;

        let func_interface = namespace.get_cur_func_interf_mut()?;
        let while_body = func_interface.new_bblock(program, "%while_body");
        let while_end = func_interface.new_bblock(program, "%while_end");
        let branch = func_interface.value_builder(program).branch(cond_val, while_body, while_end);
        func_interface.push_inst_to_bb(program, func_interface.current_bb(), branch);
        func_interface.push_bblock(program, while_body);


        namespace.set_loop_continue_break(while_entry, while_end);
        self.body_stmt.generate(namespace, program)?;
        namespace.pop_loop_continue_break();

        let func_interface = namespace.get_cur_func_interf_mut()?;
        let jump_into_while = func_interface.value_builder(program).jump(while_entry);
        func_interface.push_inst_to_bb(program, func_interface.current_bb(), jump_into_while);

        func_interface.push_bblock(program, while_end);
        return Ok(());
    }
}

impl GenerateKoopa for BreakStmt {
    type Out = ();
    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {
        let jump_to = namespace.get_break_to()?;
        let func_interface = namespace.get_cur_func_interf_mut()?;
        
        let jump = func_interface.value_builder(program).jump(jump_to);
        func_interface.push_inst_to_bb(program, func_interface.current_bb(), jump);
        let next = func_interface.new_anomynous_bblock(program);
        func_interface.push_bblock(program, next);
        return Ok(());
    }
}

impl GenerateKoopa for ContinueStmt {
    type Out = ();
    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {
        let jump_to = namespace.get_continue_to()?;
        let func_interface = namespace.get_cur_func_interf_mut()?;
        
        let jump = func_interface.value_builder(program).jump(jump_to);
        func_interface.push_inst_to_bb(program, func_interface.current_bb(), jump);
        let next = func_interface.new_anomynous_bblock(program);
        func_interface.push_bblock(program, next);
        return Ok(());
    }
}

impl GenerateKoopa for ConstExpr {
    type Out = i32;

    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {
        return self.const_eval(namespace).ok_or(CompileError::InvalidInit("".to_owned()));
    }
}

impl GenerateKoopa for Expr{
    type Out = ExprValue;

    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {
        match self {
            Self::LOr(lor) => lor.generate(namespace, program),
        }
    }
}

impl GenerateKoopa for LOrExpr{
    type Out = ExprValue;

    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {
        match self {
            Self::LAndExpr(expr) => expr.generate(namespace, program),
            Self::LOrExpr(lexp, rexp) => {
                let lv = lexp.generate(namespace, program)?.into_value(program, namespace)?;
                let func_interface = namespace.get_cur_func_interf_mut()?;
                let result = func_interface.value_builder(program).alloc(Type::get_i32());
                func_interface.push_inst_to_bb(program, func_interface.current_bb(), result);
                let ir_zero = func_interface.value_builder(program).integer(0);
                let lv = func_interface.value_builder(program).binary(BinaryOp::NotEq, lv, ir_zero);
                func_interface.push_inst_to_bb(program, func_interface.current_bb(), lv);

                let store_to_result = func_interface.value_builder(program).store(lv, result);
                func_interface.push_inst_to_bb(program, func_interface.current_bb(), store_to_result);

                let rexp_bb = func_interface.new_bblock(program, "%land_rexp");
                let end_bb = func_interface.new_bblock(program, "%land_end");
                let branch = func_interface.value_builder(program).branch(lv, end_bb, rexp_bb); // if lv!=0 to end_bb else to rexp
                func_interface.push_inst_to_bb(program, func_interface.current_bb(), branch);
                
                func_interface.push_bblock(program, rexp_bb);
                let rv = rexp.generate(namespace, program)?.into_value(program, namespace)?;

                let func_interface = namespace.get_cur_func_interf_mut()?;  // 细化namespace的可变借用作用域，避免同时存在不可变借用和可变借用
                let rv = func_interface.value_builder(program).binary(BinaryOp::NotEq, rv, ir_zero);
                func_interface.push_inst_to_bb(program, func_interface.current_bb(), rv);

                let store_to_result = func_interface.value_builder(program).store(rv, result);
                func_interface.push_inst_to_bb(program, func_interface.current_bb(), store_to_result);

                let jump = func_interface.value_builder(program).jump(end_bb);
                func_interface.push_inst_to_bb(program, func_interface.current_bb(), jump);
                func_interface.push_bblock(program, end_bb);

                let load = func_interface.value_builder(program).load(result);
                func_interface.push_inst_to_bb(program, func_interface.current_bb(), load);
                return Ok(ExprValue::VarInt(load));
            }
        }
    }
}

impl GenerateKoopa for LAndExpr{
    type Out = ExprValue;

    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {
        match self {
            Self::EqExpr(expr) => expr.generate(namespace,program),
            Self::LAndExpr(lexp, rexp) => {
                let lv = lexp.generate(namespace, program)?.into_value(program, namespace)?;
                let func_interface = namespace.get_cur_func_interf_mut()?;
                let result = func_interface.value_builder(program).alloc(Type::get_i32());
                func_interface.push_inst_to_bb(program, func_interface.current_bb(), result);
                let ir_zero = func_interface.value_builder(program).integer(0);
                let lv = func_interface.value_builder(program).binary(BinaryOp::NotEq, lv, ir_zero);
                func_interface.push_inst_to_bb(program, func_interface.current_bb(), lv);

                let store_to_result = func_interface.value_builder(program).store(lv, result);
                func_interface.push_inst_to_bb(program, func_interface.current_bb(), store_to_result);

                let rexp_bb = func_interface.new_bblock(program, "%land_rexp");
                let end_bb = func_interface.new_bblock(program, "%land_end");
                let branch = func_interface.value_builder(program).branch(lv, rexp_bb, end_bb); // if lv!=0 to rexp_bb else to end
                func_interface.push_inst_to_bb(program, func_interface.current_bb(), branch);
                
                func_interface.push_bblock(program, rexp_bb);
                let rv = rexp.generate(namespace, program)?.into_value(program, namespace)?;

                let func_interface = namespace.get_cur_func_interf_mut()?;  // 细化namespace的可变借用作用域，避免同时存在不可变借用和可变借用
                let rv = func_interface.value_builder(program).binary(BinaryOp::NotEq, rv, ir_zero);
                func_interface.push_inst_to_bb(program, func_interface.current_bb(), rv);

                let store_to_result = func_interface.value_builder(program).store(rv, result);
                func_interface.push_inst_to_bb(program, func_interface.current_bb(), store_to_result);

                let jump = func_interface.value_builder(program).jump(end_bb);
                func_interface.push_inst_to_bb(program, func_interface.current_bb(), jump);
                func_interface.push_bblock(program, end_bb);

                let load = func_interface.value_builder(program).load(result);
                func_interface.push_inst_to_bb(program, func_interface.current_bb(), load);
                return Ok(ExprValue::VarInt(load));
            }
        }
    }
}

impl GenerateKoopa for EqExpr{
    type Out = ExprValue;

    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {
        match self {
            Self::RelExpr(expr) => expr.generate(namespace, program),
            Self::EqExpr(lexp, op, rexp) => {
                let lv = lexp.generate(namespace, program)?.into_value(program, namespace)?;
                let rv = rexp.generate(namespace, program)?.into_value(program, namespace)?;
                let op = match op {
                    EqOp::Eq => BinaryOp::Eq,
                    EqOp::Ne => BinaryOp::NotEq,
                };
                let func_interface = namespace.get_cur_func_interf()?;
                let result = func_interface.value_builder(program).binary(op, lv, rv);
                func_interface.push_inst_to_bb(program, func_interface.current_bb(), result);
                return Ok(ExprValue::VarInt(result));
            }
        }
    }
}

impl GenerateKoopa for RelExpr{
    type Out = ExprValue;

    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {
        match self {
            Self::AddExpr(expr) => expr.generate(namespace, program),
            Self::RelExpr(lexp, op, rexp) => {
                let lv = lexp.generate(namespace, program)?.into_value(program, namespace)?;
                let rv = rexp.generate(namespace, program)?.into_value(program, namespace)?;
                let op = match op {
                    RelOp::Lt => BinaryOp::Lt,
                    RelOp::Gt => BinaryOp::Gt,
                    RelOp::Le => BinaryOp::Le,
                    RelOp::Ge => BinaryOp::Ge,
                };
                let func_interface = namespace.get_cur_func_interf()?;
                let result = func_interface.value_builder(program).binary(op, lv, rv);
                func_interface.push_inst_to_bb(program, func_interface.current_bb(), result);
                return Ok(ExprValue::VarInt(result));
            }
        }
    }
}

impl GenerateKoopa for AddExpr{
    type Out = ExprValue;

    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {
        match self {
            Self::MulExpr(expr) => expr.generate(namespace, program),
            Self::AddAndMul(lexp, op, rexp) => {
                let lv = lexp.generate(namespace, program)?.into_value(program, namespace)?;
                let rv = rexp.generate(namespace, program)?.into_value(program, namespace)?;
                let op = match op {
                    AddOp::Add => BinaryOp::Add,
                    AddOp::Minus => BinaryOp::Sub,
                };
                let func_interface = namespace.get_cur_func_interf()?;
                let result = func_interface.value_builder(program).binary(op, lv, rv);
                func_interface.push_inst_to_bb(program, func_interface.current_bb(), result);
                return Ok(ExprValue::VarInt(result));
            }
        }
    }
}

impl GenerateKoopa for MulExpr{
    type Out = ExprValue;

    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {
        match self {
            Self::UnaryExpr(expr) => expr.generate(namespace, program),
            Self::MulAndUnary(lexp, op, rexp) => {
                let lv = lexp.generate(namespace, program)?.into_value(program, namespace)?;
                let rv = rexp.generate(namespace, program)?.into_value(program, namespace)?;
                let op = match op {
                    MulOp::Mul => BinaryOp::Mul,
                    MulOp::Div => BinaryOp::Div,
                    MulOp::Mod => BinaryOp::Mod,
                };
                let func_interface = namespace.get_cur_func_interf()?;
                let result = func_interface.value_builder(program).binary(op, lv, rv);
                func_interface.push_inst_to_bb(program, func_interface.current_bb(), result);
                return Ok(ExprValue::VarInt(result));
            }
        }
    }
}

impl GenerateKoopa for UnaryExpr{
    type Out = ExprValue;

    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {
        match self {
            Self::FuncCall(fc) => fc.generate(namespace, program),
            Self::PrimExpr(expr) => expr.generate(namespace, program),
            Self::UnaryExpr(op, expr) => {
                let v = expr.generate(namespace, program)?.into_value(program, namespace)?;
                let func_interface = namespace.get_cur_func_interf()?;
                let ir_zero = func_interface.value_builder(program).integer(0);
                let result = match op {
                    UnaryOp::Neg => {
                        let val = func_interface.value_builder(program).binary(BinaryOp::Sub, ir_zero, v);
                        func_interface.push_inst_to_bb(program, func_interface.current_bb(), val);
                        val
                    }
                    UnaryOp::Pos => v,
                    UnaryOp::Not => {
                        let val = func_interface.value_builder(program).binary(BinaryOp::Eq, v, ir_zero);
                        func_interface.push_inst_to_bb(program, func_interface.current_bb(), val);
                        val
                    },
                };
                
                return Ok(ExprValue::VarInt(result));
            }
        }
    }
}

impl GenerateKoopa for FuncCall{
    type Out = ExprValue;

    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {
        let func_target: Function = namespace.get_func(&self.funcid)?.to_owned();
        let (params, void_ret) = match program.func(func_target).ty().kind(){
            TypeKind::Function(params, ret) => {
                (params.to_owned(), ret.is_unit())
            },
            _ => unreachable!()
        };

        let args = self
            .args
            .iter()
            .map(|arg| arg.generate(namespace, program)?.into_value_or_ptr(program, namespace))
            .collect::<CResult<Vec<Value>>>()?;

        if params.len() != args.len() {
            return Err(CompileError::InvalidFunccall("".to_owned()));
        }

        for (param_needed, arg_input) in params.iter().zip(args.iter()) {
            let arg_ty = get_type(namespace, program, *arg_input);
            if param_needed != &arg_ty {
                return Err(CompileError::InvalidFunccall("".to_owned()));
            }
        }

        let func_interface = namespace.get_cur_func_interf()?;
        let call_inst = func_interface.value_builder(program).call(func_target, args);
        func_interface.push_inst_to_bb(program, func_interface.current_bb(), call_inst);

        if void_ret {
            return Ok(ExprValue::Void);
        }
        else{
            return Ok(ExprValue::VarInt(call_inst));
        }
        
    }
}


impl GenerateKoopa for PrimExpr{
    type Out = ExprValue;

    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {
        match self {
            Self::Expr(expr) => expr.generate(namespace, program),
            Self::LVal(lval) => lval.generate(namespace, program),
            Self::Number(num) => {
                let func_interface = namespace.get_cur_func_interf()?;
                let ir_num = func_interface.value_builder(program).integer(*num);
                return Ok(ExprValue::VarInt(ir_num));
            }
        }
    }
}

impl GenerateKoopa for LVal{
    type Out = ExprValue;

    fn generate(&self, namespace: &mut Namesp, program: &mut Program) -> CResult<Self::Out> {
        let mut val = match namespace.get_value(&self.id)? {
            NamespValue::ConstInt(i) => {
                if(!self.inds.is_empty()){
                    return Err(CompileError::InvalidIdentifier("".to_owned()));
                }
                else {
                    let ir_int = namespace.get_cur_func_interf()?.value_builder(program).integer(*i);
                    return Ok(ExprValue::VarInt(ir_int));
                }
            },
            NamespValue::Var(v) => *v,
        };
        let mut is_array_param = false; // only one case: array int a[b][c] as function param int a[][c]
        let mut dims = 0;
        match get_type(namespace, program, val).kind(){
            TypeKind::Pointer(unit_ty) => {
                let mut ty = unit_ty;
                loop {
                    ty = match ty.kind(){
                        TypeKind::Array(unit_ty, _) => {
                            unit_ty
                        },
                        TypeKind::Pointer(unit_ty) => {
                            is_array_param = true;
                            unit_ty
                        },
                        _ => break,
                    };
                    dims += 1;
                }
            }
            _ => {dims = 0;},
        };
        if is_array_param{ // transform param array to array pointer
            let func_interface = namespace.get_cur_func_interf()?;
            val = func_interface.value_builder(program).load(val);
            func_interface.push_inst_to_bb(program, func_interface.current_bb(), val);
        }

        for (i, ind) in self.inds.iter().enumerate() {
            if dims == 0 {
                return Err(CompileError::InvalidArrayDeref("".to_owned()));
            }
            dims -= 1;
            let ind_int = ind.generate(namespace, program)?.into_value_or_ptr(program, namespace)?;
            let func_interface = namespace.get_cur_func_interf()?;

            val = if is_array_param && i == 0{
                func_interface.value_builder(program).get_ptr(val, ind_int)
            }
            else{
                func_interface.value_builder(program).get_elem_ptr(val, ind_int)
            };
            func_interface.push_inst_to_bb(program, func_interface.current_bb(), val);

        }
        if dims == 0 {
            return Ok(ExprValue::VarPtr(val));
        }
        else if !is_array_param || !self.inds.is_empty() { // one-dim array param
            let func_interface = namespace.get_cur_func_interf()?;
            let ir_zero = func_interface.value_builder(program).integer(0);
            val = func_interface.value_builder(program).get_elem_ptr(val, ir_zero);
            func_interface.push_inst_to_bb(program, func_interface.current_bb(), val);
            return Ok(ExprValue::ArrPtr(val));
        }
        else {
            return Ok(ExprValue::ArrPtr(val));
        }


    }
}