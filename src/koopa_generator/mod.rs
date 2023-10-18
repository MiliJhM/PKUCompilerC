mod namespace;
mod generator;
mod function_interface;

use generator::GenerateKoopa;
use crate::ast::ast_def::*;
use namespace::Namesp;
use koopa::ir::{Program, Type};
use std::fmt;

pub fn generate_program(comp_unit: &CompileInit) -> CResult<Program> {
    let mut program = Program::new();
    let mut namesp = Namesp::new();
    comp_unit.generate(&mut namesp, &mut program)?;
    Ok(program)
}

#[derive(Debug)]
pub enum CompileError{
    InvalidIdentifier(String),
    InvalidType(String),
    DuplicateIdentifier(String),
    VarNotDeclared(String),
    FuncNotDeclared(String),
}
// CResult
pub type CResult<T> = std::result::Result<T, CompileError>;