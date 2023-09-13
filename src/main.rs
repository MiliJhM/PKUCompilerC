use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{Result, Write};
use lazy_static::lazy_static;

lazy_static! {
  static ref GENERATOR: HashMap<&'static str, &'static str> = {
    let mut map = HashMap::new();
    map.insert("-koopa", r#"fun @main(): i32 {
%entry:
  ret 0
}
"#);
    map.insert("-riscv", r#"  .text
  .globl main
main:
  li a0, 0
  ret
"#);
    map.insert("-perf", r#"  .text
  .globl main
main:
  li a0, 0
  ret
"#);
    map
  };
}

fn main() -> Result<()> {
  let mut args = env::args();
  args.next();
  let mode = args.next().unwrap();
  let input = args.next().unwrap();
  assert_eq!(args.next(), Some("-o"));
  let output = args.next().unwrap();
  let mut file = File::create(output)?;
  write!(file, "{}", GENERATOR[mode.as_str()])

  
}
