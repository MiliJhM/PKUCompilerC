use std::collections::HashMap;
use std::cell::Cell;
use koopa::ir::entities::*;
use koopa::ir::{Function, BasicBlock, TypeKind, ValueKind};

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