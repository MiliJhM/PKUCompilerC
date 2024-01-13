use std::fs::File;
use std::io::{Write, Result};
use super::program_manager::*;

pub struct Writer<'file> {
    pub f: &'file mut File,
    pub reg_temp: &'static str,
    
}

impl<'file> Writer<'file> {
    pub fn new(f: &'file mut File) -> Self {
        Self {
            f,
            reg_temp: "t0",
        }
    }

    pub fn update_temp_reg(&mut self, reg: &'static str) {
        self.reg_temp = reg;
    }

    pub fn func_entry(&mut self, func_name: &str, func_interface: &FunctionInterface) -> Result<()> {
        writeln!(self.f, "  .globl {}", &func_name[1..])?;
        writeln!(self.f, "{}:", &func_name[1..])?;

        let offset = func_interface.sp_offset();
        let offset = offset as i32;
        if offset != 0 {
            self.addi("sp", "sp", -offset)?;

            if !func_interface.neednt_restore_ra(){
                self.sw("ra", "sp", offset - 4)?;
            }
        }   
        return Ok(());
    }

    pub fn func_end(&mut self, func_interface: &FunctionInterface) -> Result<()> {
        let offset = func_interface.sp_offset();
        let offset = offset as i32;
        if offset != 0 {
            if !func_interface.neednt_restore_ra(){
                self.lw("ra", "sp", offset - 4)?;
            }
            self.addi("sp", "sp", offset)?;
        }
        self.ret()?;
        return Ok(());
    }

    pub fn beqz(&mut self, cond: &str, label: &str) -> Result<()>  {
        writeln!(self.f, "  beqz {}, {}", cond, label)?;
        // self.f.write_fmt(format_args!("beqz {}, {}\n", cond, label))?;
        return Ok(());
    }

    pub fn bnez(&mut self, cond: &str, label: &str) -> Result<()>  {
        writeln!(self.f, "  bnez {}, {}", cond, label)?;
        // self.f.write_fmt(format_args!("bnez {}, {}\n", cond, label))?;
        return Ok(());
    }

    pub fn j(&mut self, label: &str) -> Result<()>  {
        writeln!(self.f, "  j {}", label)?;
        // self.f.write_fmt(format_args!("j {}\n", label))?;
        return Ok(());
    }

    pub fn call(&mut self, func: &str) -> Result<()>  {
        writeln!(self.f, "  call {}", func)?;
        // self.f.write_fmt(format_args!("call {}\n", func))?;
        return Ok(());
    }

    pub fn ret(&mut self) -> Result<()>  {
        writeln!(self.f, "  ret")?;
        // self.f.write_fmt(format_args!("ret\n"))?;
        return Ok(());
    }

    // lw rd imm12(rs)  read from rs+imm12 to rd
    pub fn lw(&mut self, rd: &str, rs: &str, offset: i32) -> Result<()>  {
        if offset <= 2047 && offset >= -2048 {
            writeln!(self.f, "  lw {}, {}({})", rd, offset, rs)?;
        }
        else {
            self.addi(&self.reg_temp, rs, offset)?;
            writeln!(self.f, "  lw {}, 0({})", rd, &self.reg_temp)?;
        }
        // self.f.write_fmt(format_args!("lw {}, {}({})\n", rd, offset, rs))?;
        return Ok(());
    }

    // sw rs imm12(rd)  store rs into imm12(rd)
    pub fn sw(&mut self, rs: &str, rd: &str, offset: i32) -> Result<()>  {
        if offset <= 2047 && offset >= -2048 {
            writeln!(self.f, "  sw {}, {}({})", rs, offset, rd)?;
        }
        else {
            self.addi(&self.reg_temp, rd, offset)?;
            writeln!(self.f, "  sw {}, 0({})", rs, &self.reg_temp)?;
        }
        // self.f.write_fmt(format_args!("sw {}, {}({})\n", rs, offset, rd))?;
        return Ok(());
    }

    // op rd rs1 rs2: add\sub\slt\sgt\xor\or\and\sll\srl\sra\mul\div\rem
    pub fn op2(&mut self, op: &str, rd: &str, rs1: &str, rs2: &str) -> Result<()>  {
        writeln!(self.f, "  {} {}, {}, {}", op, rd, rs1, rs2)?;
        // self.f.write_fmt(format_args!("{} {}, {}, {}\n", op, rd, rs1, rs2))?;
        return Ok(());
    }

    // op2i: Only support add\sub\xor\or\and
    // op rd rs imm
    pub fn op2i(&mut self, op: &str, rd: &str, rs1: &str, imm: i32) -> Result<()>  {

        if imm <= 2047 && imm >= -2048 {
            writeln!(self.f, "  {}i {}, {}, {}", op, rd, rs1, imm)?;
        }
        else {
            self.li(&self.reg_temp, imm)?;
            self.op2(op, rd, rs1, &self.reg_temp)?;
        }
        // self.f.write_fmt(format_args!("{} {}, {}, {}\n", op, rd, rs1, imm))?;
        return Ok(());
    }

    pub fn addi(&mut self, rd: &str, rs1: &str, imm: i32) -> Result<()>  {
        self.op2i("add", rd, rs1, imm)?;
        // self.f.write_fmt(format_args!("addi {}, {}, {}\n", rd, rs1, imm))?;
        return Ok(());
    }

    pub fn muli(&mut self, rd: &str, rs1: &str, imm: i32) -> Result<()>  {
        if imm == 0 {
            self.mv(rd, "x0")?;
        }
        else if imm > 0 && (imm & (imm-1)) == 0{
            let mut shift = 0;
            let mut imm = imm>>1;
            while imm != 0 {
                imm = imm>>1;
                shift += 1;
            }
            self.op2i("sll", rd, rs1, shift)?;
        }
        else {
            self.li(&self.reg_temp, imm)?;
            self.op2("mul", rd, rs1, &self.reg_temp)?;
        }
        return Ok(());
    }

    // op1 rd rs1: seqz/snez
    pub fn op1(&mut self, op: &str, rd: &str, rs1: &str) -> Result<()>  {
        writeln!(self.f, "  {} {}, {}", op, rd, rs1)?;
        // self.f.write_fmt(format_args!("{} {}, {}\n", op, rd, rs1))?;
        return Ok(());
    }

    // li rd imm
    pub fn li(&mut self, rd: &str, imm: i32) -> Result<()>  {
        writeln!(self.f, "  li {}, {}", rd, imm)?;
        // self.f.write_fmt(format_args!("li {}, {}\n", rd, imm))?;
        return Ok(());
    }

    // la rd label
    pub fn la(&mut self, rd: &str, label: &str) -> Result<()>  {
        writeln!(self.f, "  la {}, {}", rd, label)?;
        // self.f.write_fmt(format_args!("la {}, {}\n", rd, label))?;
        return Ok(());
    }

    // mv rd rs
    pub fn mv(&mut self, rd: &str, rs: &str) -> Result<()>  {
        writeln!(self.f, "  mv {}, {}", rd, rs)?;
        // self.f.write_fmt(format_args!("mv {}, {}\n", rd, rs))?;
        return Ok(());
    }

    pub fn file_mut(&mut self) -> &mut File {
        self.f
    }
}
