use crate::ast::ast_def::*;
use super::{CResult, CompileError, function_interface::{FunctionInterface, self}};
use std::{collections::HashMap, hash::Hash};
use koopa::ir::{*, builder_traits::{LocalInstBuilder, ValueBuilder}};

pub enum NamespValue{
    ConstInt(i32),
    Var(Value),
}

pub enum ExprValue{
    Void,           // Void
    VarInt(Value),  // Var
    VarPtr(Value),  // Pointer
    ArrPtr(Value),  // Array
}

pub struct Namesp{
    value_maps: Vec<HashMap<String, NamespValue>>, // Stack
    funcs: HashMap<String, Function>, // Global Function Table
    is_const: Vec<HashMap<String, bool>>, // Const Bool Stack
    pub cur_function: Option<FunctionInterface>,
    //cur_func_ret: Option<Value>,

    pub continue_break_stack: Vec<(BasicBlock, BasicBlock)>,
}

impl Namesp{
    pub fn new() -> Self {
        Self{
            value_maps: vec![HashMap::new()],
            funcs: HashMap::new(),
            is_const: vec![HashMap::new()],
            cur_function: None,
            //cur_func_ret: None,
            continue_break_stack: Vec::new(),
        }
    }

    // Core: Stack Implementation of Namespace Layers
    pub fn enter_new_scope(&mut self) {
        self.value_maps.push(HashMap::new());
        self.is_const.push(HashMap::new());
    }

    pub fn exit_now_scope(&mut self) {
        self.value_maps.pop();
        self.is_const.pop();
        if self.value_maps.len() == 1 { // Global, Exit Function
            self.cur_function = None;
            //self.cur_func_ret = None;
        }
    }

    // Create New: Value or Function
    pub fn new_value(&mut self, var_id:&str, var_value: NamespValue, if_const: bool) -> CResult<()> {
        let global_var: bool = self.value_maps.len() == 1;
        let cur_layer: &mut HashMap<String, NamespValue> = self.value_maps.last_mut().unwrap();
        if cur_layer.contains_key(var_id) {
            return Err(CompileError::DuplicateIdentifier(var_id.to_owned()));
        }
        if global_var && self.funcs.contains_key(var_id) {
            return Err(CompileError::DuplicateIdentifier(var_id.to_owned()));
        }
        cur_layer.insert(var_id.to_string(), var_value);
        self.is_const.last_mut().unwrap().insert(var_id.to_string(), if_const);
        return Ok(());
    }

    pub fn new_func(&mut self, func_id: &str, func_def: Function) -> CResult<()> {
        if self.funcs.contains_key(func_id) || self.value_maps[0].contains_key(func_id) {
            return Err(CompileError::DuplicateIdentifier(func_id.to_owned()));
        }
        self.funcs.insert(func_id.to_string(), func_def);
        //self.cur_function = Some(function_interface);
        return Ok(());
    }
    // Get Value: Value or Function
    pub fn get_value(&self, var_id: &str) -> CResult<&NamespValue> {
        for layer in self.value_maps.iter().rev() {
            if let Some(value) = layer.get(var_id) {
                return Ok(value);
            }
        }
        return Err(CompileError::VarNotDeclared(var_id.to_owned()));
    }

    pub fn get_func(&self, func_id: &str) -> CResult<&Function> {
        if let Some(func) = self.funcs.get(func_id) {
            return Ok(func);
        }
        return Err(CompileError::FuncNotDeclared(func_id.to_owned()));
    }

    pub fn get_cur_func_interf(&self) -> CResult<&FunctionInterface> {
        if let Some(func) = &self.cur_function {
            return Ok(func);
        }
        return Err(CompileError::FuncNotDeclared("".to_owned())); // TODO: Error Message
    }

    pub fn get_cur_func_interf_mut(&mut self) -> CResult<&mut FunctionInterface> {
        if let Some(func) = &mut self.cur_function {
            return Ok(func);
        }
        return Err(CompileError::FuncNotDeclared("".to_owned())); // TODO: Error Message
    }

    pub fn get_continue_to(&self) -> CResult<BasicBlock> {
        if let Some((bb, _)) = self.continue_break_stack.last() {
            return Ok(*bb);
        }
        return Err(CompileError::InvalidType("".to_owned())); // TODO: Error Message
    }

    pub fn get_break_to(&self) -> CResult<BasicBlock> {
        if let Some((_, bb)) = self.continue_break_stack.last() {
            return Ok(*bb);
        }
        return Err(CompileError::InvalidType("".to_owned())); // TODO: Error Message
    }

    pub fn set_loop_continue_break(&mut self, continue_to: BasicBlock, break_to: BasicBlock) {
        self.continue_break_stack.push((continue_to, break_to));
    }

    pub fn pop_loop_continue_break(&mut self) {
        self.continue_break_stack.pop();
    }

    pub fn is_global(&self) -> bool {
        return self.value_maps.len() == 1;
    }
}
/* 
impl NamespValue {

}
*/

#[derive(Debug, Clone)]
pub enum InitValue{
    Const(i32),
    List(Vec<InitValue>),
    Var(Value),
}

impl InitValue {
    pub fn get_array_shape(&self, mut ty: &Type) -> CResult<Vec<(usize, usize)>>{
        let mut shape = Vec::new();
        loop {
            match ty.kind() {
                TypeKind::Array(unit_ty, len) => {
                    shape.push(*len);
                    ty = unit_ty;
                },
                TypeKind::Int32 => break,
                _ => unreachable!(),
            }
        }
        let mut temp_len = 1; 
        let shape_ast = shape.into_iter().rev().map(  |len| {temp_len *= len; return (len, temp_len);}  ).collect();
        return Ok(shape_ast);
        // shape_ast: [a][b][c] -> [(c,c),(b,b*c),(a,a*b*c)]
    }

    pub fn array_linearize(&self, ty: &Type) -> CResult<Vec<InitValue>> {
        println!("before linearize: {:?}", &self);
        let mut result = Vec::new();
        let mut shape = self.get_array_shape(ty)?;
        dbg!(shape.clone());
        shape.reverse();
        
        let mut dim_len = shape.len();
        let mut init_num = 0;
        let mut init_needed = shape[0].1;
        
        match self {
            Self::List(init_vals) => {
                for init_val in init_vals {
                    if init_num >= init_needed {
                        return Err(CompileError::InvalidInit("".to_owned()));
                    }
                    match init_val {
                        Self::Const(value) => {
                            result.push(Self::Const(*value));
                            init_num+=1;
                        },
                        Self::Var(value) => {
                            result.push(Self::Var(*value));
                            init_num+=1;
                        },
                        Self::List(init_vals) => {
                            let mut child_type = ty.clone();
                            let mut flag = false;
                            for i in 1..dim_len {
                                dbg!(init_num, shape[i].1, init_num % shape[i].1);
                                match child_type.kind() {
                                    TypeKind::Array(unit_ty, _) => {
                                        child_type = unit_ty.clone();
                                    },
                                    _ => unreachable!(),
                                }
                                if init_num % shape[i].1 == 0 {
                                    result.append(&mut init_val.array_linearize(&child_type)?);
                                    init_num+=shape[i].1;
                                    flag = true;
                                    break;
                                }
                            }
                            if !flag {
                                return Err(CompileError::InvalidInit("".to_owned()));
                            }
                        }
                    }
                }
            },
            _ => unreachable!(),
        }
        while init_num < init_needed {
            result.push(Self::Const(0));
            init_num+=1;
            
        }
        println!("after linearize: {:?}", &result);
        return Ok(result);
    }

    pub fn init_rebuild(&self, ty: &Type) -> CResult<InitValue> {
        let result = match self {
            Self::Const(value) => Ok(Self::Const(*value)),
            Self::Var(value) => Ok(Self::Var(*value)),
            Self::List(init_vals) => {
                let shape = self.get_array_shape(ty)?;
                let mut linear_result = self.array_linearize(ty)?;
                for i in 0..shape.len()-1 {
                    let len = shape[i].0;
                    let mut temp_result = Vec::new();
                    let mut new_list = Vec::new();
                    for (i, l) in linear_result.iter().enumerate(){
                        temp_result.push(l.clone());
                        if (i+1) % len == 0 {
                            new_list.push(Self::List(temp_result.clone()));
                            temp_result.clear();
                        }
                    }
                    linear_result = new_list;
                }
                Ok(Self::List(linear_result))
            },
        };
        return Ok(result?);
    }

    pub fn into_const(self, program: &mut Program, namespace: &mut Namesp) -> CResult<Value> {
        match self {
            Self::Const(value) => Ok(
                if namespace.is_global()
                {program.new_value().integer(value)} 
                else {namespace.get_cur_func_interf_mut()?.value_builder(program).integer(value)}),
            Self::Var(_) => Err(CompileError::InvalidType("".to_owned())),

            Self::List(init_vals) => {
                let values = init_vals.into_iter().map(|init_val| init_val.into_const(program, namespace)).collect::<CResult<Vec<Value>>>()?;

                let ret = if namespace.is_global() {
                    program.new_value().aggregate(values)
                } else {
                    namespace.get_cur_func_interf_mut()?.value_builder(program).aggregate(values)
                };
                Ok(ret)
            },

        }
    }

    pub fn into_ptr_stored(self, program: &mut Program, namespace: & Namesp, ptr: Value){
        let func_interface = namespace.get_cur_func_interf().unwrap();
        match self {
            Self::Const(integer) => {
                let value = func_interface.value_builder(program).integer(integer);
                let value = func_interface.value_builder(program).store(value, ptr);
                func_interface.push_inst_to_bb(program, func_interface.current_bb(), value);
            },
            Self::Var(value) => {
                let value = func_interface.value_builder(program).store(value, ptr);
                func_interface.push_inst_to_bb(program, func_interface.current_bb(), value);
            },
            Self::List(init_vals) => {
                for (i, init_val) in init_vals.into_iter().enumerate() {
                    let ind = func_interface.value_builder(program).integer(i as i32);
                    let target_ptr = func_interface.value_builder(program).get_elem_ptr(ptr, ind);
                    func_interface.push_inst_to_bb(program, func_interface.current_bb(), target_ptr);
                    init_val.into_ptr_stored(program, namespace, target_ptr);
                }
            },
        };
        return;
    }
} 

impl ExprValue{
    pub fn get_value(self, program: &mut Program, namespace: &mut Namesp) -> CResult<Value>{
        match self {
            Self::Void => Err(CompileError::InvalidType("".to_owned())),
            Self::VarInt(value) => Ok(value),
            Self::VarPtr(ptr) => {
                let mut func_interface = namespace.get_cur_func_interf()?;
                let mut load_inst = func_interface.value_builder(program).load(ptr);
                func_interface.push_inst_to_bb(program, func_interface.current_bb(), load_inst);
                Ok(load_inst)
            }
            Self::ArrPtr(value) => Ok(value),
        }
    }

    pub fn into_integer(self, program: &mut Program, namespace: &mut Namesp) -> CResult<Value> {
        match self {
            Self::VarInt(value) => Ok(value),
            Self::VarPtr(value) => {
                let mut func_interface = namespace.get_cur_func_interf()?;
                let mut load_inst = func_interface.value_builder(program).load(value);
                func_interface.push_inst_to_bb(program, func_interface.current_bb(), load_inst);
                Ok(load_inst)
            },
            _ => Err(CompileError::InvalidType("".to_owned())),
        }
    }

    pub fn into_lvptr(self)->CResult<Value>{
        match self{
            Self::VarPtr(value) => Ok(value),
            _ => Err(CompileError::InvalidType("".to_owned())),
        }
    }

}