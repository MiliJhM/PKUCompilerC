use koopa::ir::entities::*;
use koopa::ir::{Function, BasicBlock, TypeKind, ValueKind};
use std::collections::HashMap;
use std::cell::Cell;


pub struct ProgramManager<'prog> {
    program: &'prog Program,
    functions: HashMap<String, Function>,
    values_names: HashMap<Value, String>,

    current_function: Option<FunctionInterface>,
    current_block: Option<BasicBlock>,
    current_function_name: Option<String>,

    /* 
    registers: HashMap<String, Register>,
    address: HashMap<String, Address>,
    */
}

impl<'prog> ProgramManager<'prog>{
    pub fn new(program: &'prog Program) -> Self{
        Self{
            program,
            functions: HashMap::new(),
            values_names: HashMap::new(),
            current_function: None,
            current_block: None,
            current_function_name: None,
        }


    }
    pub fn program(&self) -> &'prog Program{
        self.program
    }
    
    pub fn value_name(&self, value: Value) -> &String{
        self.values_names.get(&value).unwrap()
    }

    pub fn insert_value(&mut self, value: Value, name: String){
        self.values_names.insert(value, name);
    }

    pub fn set_cur_func(&mut self, func: FunctionInterface){
        self.current_function = Some(func);
    }

    pub fn cur_func(&self) -> Option<&FunctionInterface>{
        self.current_function.as_ref()
    }

    pub fn cur_func_mut(&mut self) -> Option<&mut FunctionInterface>{
        self.current_function.as_mut()
    }

}  


pub struct FunctionInterface{
    func: Function,
    max_arg_num: Option<usize>,
    
    allocated_stacksize: usize,
    allocated: HashMap<*const ValueData, ValueSlot>,

    bb_names: HashMap<BasicBlock, String>,
    stackp_offset: Cell<Option<usize>>,

    
}

impl FunctionInterface{

    thread_local! {
        static NEXT_TEMP_LABEL_ID: Cell<usize> = Cell::new(0);
    }

    
    pub fn new(func: Function) -> Self{
        Self{
            func,
            max_arg_num: None,
            allocated_stacksize: 0,
            allocated: HashMap::new(),
            bb_names: HashMap::new(),
            stackp_offset: Cell::new(None),
        }
    }

    pub fn get_func(&self) -> Function{
        self.func
    }

    pub fn update_max_arg_num(&mut self, num: usize){
        if self.max_arg_num.is_none() || num > self.max_arg_num.unwrap() {
            self.max_arg_num = Some(num);
        }
    }

    pub fn get_arg_num(&self) -> Option<usize>{
        self.max_arg_num
    }

    pub fn neednt_restore_ra(&self) -> bool{
        self.max_arg_num.is_none()
    }

    // get the final stack sp offset from the frame base
    // there the frame end with max num of args of callees
    pub fn sp_offset(&self) -> usize{
        if let Some(offset) = self.stackp_offset.get(){
            offset
        }
        else
        {
            let return_address_size = if self.neednt_restore_ra() {0} else {4};
            let arg_size = match self.max_arg_num{
                Some(num) => if num <= 8 {0} else {(num-8)*4},
                None => 0,
            };
            let offset = return_address_size+arg_size+self.allocated_stacksize;
            let final_offset = (offset+15) / 16 * 16;
            self.stackp_offset.set(Some(final_offset));
            return final_offset;
        }
    }

    pub fn set_bb_name(&mut self, bb: BasicBlock, name: &Option<String>){
        let id = Self::NEXT_TEMP_LABEL_ID.with(|id| {
            id.replace(id.get()+1)
        });
        let name = match name{
            Some(name) => format!(".L{}_index_{}", &name.clone()[1..], id),
            None => format!(".L{}", id),
        };
        self.bb_names.insert(bb, name);
        
    }

    pub fn get_bb_name(&self, bb: BasicBlock) -> &str{
        self.bb_names.get(&bb).unwrap()
    }

    pub fn alloc_new_slot(&mut self, value: &ValueData) {
        match value.kind() {
            ValueKind::Alloc(_) => {

                let slot = ValueSlot::new_stackslot(self.allocated_stacksize, false);
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
                let slot = ValueSlot::new_stackslot(self.allocated_stacksize, is_ptr);  // TODO: Register Manager to allocate register 如何管理局部变量的寄存器分配？
                self.allocated_stacksize += value.ty().size();
                self.allocated.insert(value, slot);
            }
        }
    }



    // * stack_offset_resize - 用于将相对函数入口的栈偏移量转换为相对栈指针的栈偏移量，避免内部变量暴露
    pub fn stack_offset_resize(&self, value: &ValueData) -> Option<ValueSlot> {


        match self.allocated.get(&(value as *const ValueData)) {
            Some(val) => {

                if val.get_stackslot().is_none(){
                    return None;
                }                

                let mut new_slot;
                if self.neednt_restore_ra(){
                    new_slot = Some(ValueSlot::new_stackslot(self.sp_offset() + val.stackslot_offset().unwrap() - self.allocated_stacksize , val.is_ptr()));
                }
                else {
                    new_slot = Some(ValueSlot::new_stackslot(self.sp_offset() + val.stackslot_offset().unwrap() - 4 - self.allocated_stacksize, val.is_ptr()));
                }
                dbg!(&val);
                dbg!(&new_slot);
                new_slot
            }
            None => None,
        }
        
    }
}


#[derive(Clone, Debug)]
pub struct ValueSlot{
    pub reg: Option<RegSlot>,
    pub stack: Option<StackSlot>,
    pub ptr_flag: bool,
}

impl ValueSlot{
    fn new() -> Self{
        Self{
            reg: None,
            stack: None,
            ptr_flag: false,
        }
    }

    pub fn is_ptr(&self) -> bool{
        return self.ptr_flag;
    }

    pub fn new_stackslot(offset:usize, is_ptr:bool) -> Self{
        Self{
            reg: None,
            stack: Some(StackSlot::new(offset)),
            ptr_flag: is_ptr,
        }
    }

    pub fn new_regslot(reg:String, is_ptr:bool) -> Self{
        Self{
            reg: Some(RegSlot::new(reg)),
            stack: None,
            ptr_flag: is_ptr,
        }
    }

    pub fn add_regslot(&mut self, reg:String, is_ptr:bool){
        self.reg = Some(RegSlot::new(reg));
    }

    pub fn add_stackslot(&mut self, offset:usize, is_ptr:bool){
        self.stack = Some(StackSlot::new(offset));
    }

    pub fn get_regslot(&self) -> Option<&RegSlot>{
        if let Some(reg) = &self.reg{
            Some(reg)
        }
        else {
            None
        }
    }

    pub fn get_stackslot(&self) -> Option<&StackSlot>{
        if let Some(stack) = &self.stack{
            Some(stack)
        }
        else {
            None
        }
    }

    pub fn get_regslot_mut(&mut self) -> Option<&mut RegSlot>{
        if let Some(reg) = &mut self.reg{
            Some(reg)
        }
        else {
            None
        }
    }

    pub fn get_stackslot_mut(&mut self) -> Option<&mut StackSlot>{
        if let Some(stack) = &mut self.stack{
            Some(stack)
        }
        else {
            None
        }
    }

    pub fn stackslot_offset(&self) -> Option<usize>{
        if let Some(stack) = &self.stack{
            Some(stack.offset)
        }
        else {
            None
        }
    }
}
#[derive(Clone, Debug)]
pub struct RegSlot{
    pub reg: String,
}

impl RegSlot{
    fn new(reg:String) -> Self{
        Self{
            reg,
        }
    }

    fn map(self, f: impl FnOnce(String) -> String) -> Self{
        Self{
            reg: f(self.reg),
        }
    }
}
#[derive(Clone, Debug)]
pub struct StackSlot{
    pub offset:usize,
}

impl StackSlot{
    fn new(offset:usize) -> Self{
        Self{
            offset,
        }
    }

    fn map(self, f: impl FnOnce(usize) -> usize) -> Self{
        Self{
            offset: f(self.offset),
        }
    }
}