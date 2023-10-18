use std::collections::HashMap;

use koopa::ir::{builder::LocalBuilder, builder_traits::*};
use koopa::ir::{*, dfg::*, layout::*};


pub struct FunctionInterface {
    func: Function,
    ret_value: Option<Value>,
    bbs_list: HashMap<String, BasicBlock>,
    cur_bb: Option<BasicBlock>,
}

impl FunctionInterface {
    pub fn new(func: Function, ret_value:Option<Value>) -> Self {
        Self{
            func, ret_value, bbs_list: HashMap::new(), cur_bb: None,
        }
    }

    pub fn get_func(&self) -> Function {
        self.func
    }

    pub fn get_func_data<'a>(&mut self, program: &'a mut Program) -> &'a mut FunctionData{
        return program.func_mut(self.func);
    }

    pub fn get_ret(&self) -> Option<Value> {
        self.ret_value
    }

    pub fn get_dfg_mut<'a>(&self, program: &'a mut Program) -> &'a mut DataFlowGraph {
        return program.func_mut(self.func).dfg_mut()
    }

    pub fn get_lot_mut<'a>(&self, program: &'a mut Program) -> &'a mut Layout {
        return program.func_mut(self.func).layout_mut()
    }

    pub fn get_bblock_from_list(&self, block_name: &str) -> BasicBlock {
        let bbname = block_name.to_string();
        return self.bbs_list.get(&bbname).unwrap().clone();
    }

    pub fn get_bblock_from_bb<'a>(&self, program: &'a mut Program, bb: BasicBlock) -> &'a mut BasicBlockNode {
        return self.get_lot_mut(program).bb_mut(bb);
    }

    pub fn get_bblock_from_name<'a>(&self, program: &'a mut Program, block_name: &str) -> &'a mut BasicBlockNode {
        let layout = self.get_lot_mut(program);
        //if(bbs_list.contains_key(bbname)) {
            return layout.bb_mut(self.get_bblock_from_list(block_name));
        //}
        //else{
            // TODO: Error Message
        //}

    }

    pub fn new_bblock(&mut self, program: &mut Program, block_name: &str) -> BasicBlock {
        let dfg = self.get_dfg_mut(program);
        let bbname = block_name.to_string();
        let new_bb = dfg.new_bb().basic_block(Some(bbname.clone()));
        self.bbs_list.insert(bbname, new_bb);
        return new_bb;
    }

    pub fn value_builder<'a>(&self, program: &'a mut Program) -> LocalBuilder<'a> {
        let dfg = self.get_dfg_mut(program);
        return dfg.new_value();
    }

    pub fn push_bblock(&mut self, program: &mut Program, bb: BasicBlock) {
        self.cur_bb = Some(bb);
        let layout = self.get_lot_mut(program);
        layout.bbs_mut().push_key_back(bb);
    }

    pub fn push_inst_to_bb(&mut self, program: &mut Program, bb: BasicBlock, inst: Value) {
        let layout = self.get_lot_mut(program);
        layout.bb_mut(bb).insts_mut().push_key_back(inst);
    }

    pub fn current_bb(&self) -> BasicBlock {
        let ref cur_bb = self.cur_bb;
        return cur_bb.unwrap();
    }

    
}