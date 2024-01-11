use koopa::ir::entities::*;
use koopa::ir::{Function, BasicBlock, TypeKind, ValueKind};
use std::collections::HashMap;

pub struct IRManager {
    program: Program,
    functions: HashMap<String, Function>,
    current_function: Option<Function>,
    current_block: Option<BasicBlock>,
    current_function_name: Option<String>,

    /* 
    registers: HashMap<String, Register>,
    address: HashMap<String, Address>,
    */
}

pub struct FunctionInterface{
    func: Function,
    max_arg_num: Option<usize>,
    
    allocated_stacksize: usize,
    allocated: HashMap<*const ValueData, Slot>,

    bb_names: HashMap<BasicBlock, String>,
    stackp_offset: Option<usize>,
}

impl FunctionInterface{
    pub fn new(func: Function) -> Self{
        Self{
            func,
            max_arg_num: None,
            allocated_stacksize: 0,
            allocated: HashMap::new(),
            bb_names: HashMap::new(),
            stackp_offset: None,
        }
    }

    pub fn get_func(&self) -> Function{
        self.func
    }

    pub fn update_max_arg_num(&mut self, num: usize){
        if let Some(max) = self.max_arg_num{
            if max < num{
                self.max_arg_num = Some(num);
            }
        }else if num != 0{
            self.max_arg_num = Some(num);
        }
    }

    pub fn is_no_arg(&self) -> bool{
        self.max_arg_num.is_none()
    }

    pub fn alloc_new_slot(&mut self, value: &ValueData) {
        match value.kind() {
            ValueKind::Alloc(_) => {

                let slot = Slot::new_stackslot(self.allocated_stacksize, false);
                self.allocated_stacksize += match value.ty().kind() {
                    TypeKind::Pointer(unit) => unit.size(),
                    _ => unreachable!(),
                };
                self.allocated.insert(value, slot);
            }
            _ => {
                let is_ptr = match value.ty().kind() {
                    TypeKind::Pointer(_) => true,
                    _ => false,
                };
                let slot = Slot::new_stackslot(self.allocated_stacksize, is_ptr);  // TODO: Register Manager to allocate register 如何管理局部变量的寄存器分配？
                self.allocated_stacksize += value.ty().size();
                self.allocated.insert(value, slot);
            }
        }
    }
}

pub struct Register {
    name: String,
    storing: Vec<*const ValueData>,
}

impl Register {
    pub fn new(name: String) -> Self {
        Self {
            name,
            storing: Vec::new(),
        }
    }

    pub fn store(&mut self, value: *const ValueData) {
        self.storing.push(value);
    }

    pub fn clear(&mut self) {
        self.storing.clear();
    }

    pub fn delete(&mut self, value: *const ValueData) {
        self.storing.retain(|&x| x != value);
    }
}

pub struct Slot{
    pub reg: Option<RegSlot>,
    pub stack: Option<StackSlot>,
}

impl Slot{
    fn new() -> Self{
        Self{
            reg: None,
            stack: None,
        }
    }

    fn new_stackslot(offset:usize, is_ptr:bool) -> Self{
        Self{
            reg: None,
            stack: Some(StackSlot::new(offset, is_ptr)),
        }
    }

    fn new_regslot(reg:String, is_ptr:bool) -> Self{
        Self{
            reg: Some(RegSlot::new(reg, is_ptr)),
            stack: None,
        }
    }

    fn add_regslot(&mut self, reg:String, is_ptr:bool){
        self.reg = Some(RegSlot::new(reg, is_ptr));
    }

    fn add_stackslot(&mut self, offset:usize, is_ptr:bool){
        self.stack = Some(StackSlot::new(offset, is_ptr));
    }

    fn get_regslot_mut(&mut self) -> Option<&mut RegSlot>{
        if let Some(reg) = &mut self.reg{
            Some(reg)
        }
        else {
            None
        }
    }

    fn get_stackslot_mut(&mut self) -> Option<&mut StackSlot>{
        if let Some(stack) = &mut self.stack{
            Some(stack)
        }
        else {
            None
        }
    }
}

pub struct RegSlot{
    pub reg: String,
    pub is_ptr:bool,
}

impl RegSlot{
    fn new(reg:String, is_ptr:bool) -> Self{
        Self{
            reg,
            is_ptr,
        }
    }

    fn map(self, f: impl FnOnce(String) -> String) -> Self{
        Self{
            reg: f(self.reg),
            is_ptr: self.is_ptr,
        }
    }
}

pub struct StackSlot{
    pub offset:usize,
    pub is_ptr:bool,
}

impl StackSlot{
    fn new(offset:usize, is_ptr:bool) -> Self{
        Self{
            offset,
            is_ptr,
        }
    }

    fn map(self, f: impl FnOnce(usize) -> usize) -> Self{
        Self{
            offset: f(self.offset),
            is_ptr: self.is_ptr,
        }
    }
}