use crate::ast::ast_def::*;
use super::{CResult, CompileError, function_interface::{FunctionInterface, self}};
use std::{collections::HashMap, hash::Hash};
use koopa::ir::*;

pub enum NamespValue{
    Const(i32), // Instant
    Var(Value), // Pointer
}

pub struct Namesp{
    value_maps: Vec<HashMap<String, NamespValue>>, // Stack
    funcs: HashMap<String, Function>, // Global Function Table
    is_const: Vec<HashMap<String, bool>>, // Const Bool Stack
    pub cur_function: Option<FunctionInterface>,
    //cur_func_ret: Option<Value>,
}

impl Namesp{
    pub fn new() -> Self {
        Self{
            value_maps: vec![HashMap::new()],
            funcs: HashMap::new(),
            is_const: vec![HashMap::new()],
            cur_function: None,
            //cur_func_ret: None,
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

    pub fn get_cur_func_interf(&mut self) -> CResult<&FunctionInterface> {
        if let Some(func) = &self.cur_function {
            return Ok(func);
        }
        return Err(CompileError::FuncNotDeclared("".to_owned())); // TODO: Error Message
    }
}

impl NamespValue {
    pub fn convert_from_InitVal(&mut self, val: &InitVal) -> CResult<(&mut Self)> {
        // TODO: Finish this:)    Need to calculate InitVal Expr; Need to calculate InitVal Array; Need Typecheck;
        return Ok(self);
    }
}