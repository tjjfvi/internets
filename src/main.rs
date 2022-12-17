mod parser;
mod program;
mod runtime;

use std::{env, fs};

use parser::parse;
use program::build_program;
use runtime::Runtime;

fn main() {
  let args: Vec<_> = env::args().collect();
  let program = build_program(&parse(&fs::read_to_string(&args[1]).unwrap()));
  dbg!(&program);
  let mut runtime = Runtime::new(&program);
  while runtime.has_work() {
    // dbg!(&runtime);
    runtime.step();
  }
  dbg!(&runtime);
  dbg!(runtime.dbg_tree(runtime.get(0)));
}
