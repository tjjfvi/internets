use internets_nets::*;

mod stdlib;

interactions! {
  type FibRecursive;

  use stdlib::Std;

  struct Fib(-U64,+U64);
  impl Fib(_, o) for Std::U64(_, $n @ (0|1)){ Std::U64(o, $n) }
  impl Fib(_, o) for Std::U64(_, $n) {
    Std::Add(f1,f2,o)
    Fib(n1,f1)
    Fib(n2,f2)
    Std::U64(n1, $n-1)
    Std::U64(n2, $n-2)
  }

  fn main() {
    Std::U64(n, $40)
    Fib(n, o)
    Std::Print(o)
  }
}

fn main() {
  let mut stats = Stats::default();
  let mut net = BasicNet::new(LinkAlloc::new(ArrayBuffer::new(1 << 20)));
  FibRecursive::main(&mut net);
  reduce_with_stats(&mut net, &FibRecursive, &mut stats);
  println!("{stats}");
}
