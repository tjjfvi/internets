mod parser;
mod program;
mod runtime;

use parser::parse;
use program::build_program;
use runtime::Runtime;

fn main() {
  let program = build_program(&parse(include_str!("../nets/nat.in")));
  dbg!(&program);
  let mut runtime = Runtime::new(&program);
  while runtime.has_work() {
    runtime.step();
  }
  dbg!(&runtime);
}
