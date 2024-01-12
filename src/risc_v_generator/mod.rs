pub mod code_generator;
mod program_manager;
mod asm_generator;
mod asm_value;
mod reg_manager;

use koopa::ir::{Program, Type};
use std::fs::File;
use std::io::Result;
use code_generator::AsmGenerator;
use program_manager::ProgramManager;
use asm_generator::Writer;

pub fn generate_asm(program: &Program, path: &str) -> Result<()> {
    let mut file = File::create(path)?;
    let mut writer = Writer::new(&mut file);
    let mut program_manager = ProgramManager::new(program);
    program.generate(&mut program_manager, &mut writer)?;
    return Ok(());
}