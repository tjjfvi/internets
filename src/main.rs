mod parser;
mod program;
mod runtime;

use std::{env, fs};

use crate::{parser::parse, program::build_program, runtime::Runtime};

fn main() {
  let args: Vec<_> = env::args().collect();
  let program = build_program(&parse(&fs::read_to_string(&args[1]).unwrap()));
  dbg!(&program);
  let mut runtime = Runtime::new(&program);
  while !runtime.normal() {
    // dbg!(&runtime);
    runtime.reduce();
  }
  dbg!(&runtime);
  dbg!(runtime.dbg_tree(runtime.get(runtime.free_port(0))));
}
