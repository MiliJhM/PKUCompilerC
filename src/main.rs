use std::env;
use std::fs::read_to_string;
use std::io::Result;
use std::{fmt, io};

mod ast;
mod koopa_generator;
mod risc_v_generator;
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
    // println!("{:?}", comp_init);
    let program = koopa_generator::generate_program(&comp_init).unwrap();

    let mode = match mode.as_str() {
        "-koopa" => {
            KoopaGenerator::from_path(output.as_str())?.generate_on(&program)?;
        },
        "-riscv" => {
            risc_v_generator::generate_asm(&program, output.as_str())?;
        },
        "-perf" => {
            risc_v_generator::generate_asm(&program, output.as_str())?;
        },
        _ => unreachable!("Invalid Args"),
    };
    
    return Ok(());
}

