use std::collections::HashMap;
use std::env;
use std::fs::read_to_string;
use std::fs::File;
use std::io::{Result, Write};

use crate::ast::ast_def::*;
use crate::ast::grammar;
use crate::koopa_generator::*;
use koopa::back::KoopaGenerator;

fn main() -> Result<()> {
  // Arguments Praser: mode input -o output

    let mut args = env::args();
    args.next();
    let mode = args.next().unwrap();
    println!("{}", mode);
    let input = args.next().unwrap();
    println!("{}", input);

    assert_eq!(args.next(), Some("-o".to_owned()));
    let output = args.next().unwrap(); 
    println!("{}", output);
    let input = read_to_string(input)?;
    let comp_init = ast::grammar::CompileInitParser::new().parse(&input);
    let comp_init = comp_init.unwrap();
    let mut program = koopa_generator::generate_program(&comp_init).unwrap();
    KoopaGenerator::from_path(output).unwrap().generate_on(&mut program).unwrap();
    
    return Ok(());
}
