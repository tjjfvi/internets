use internets_nets::*;

mod stdlib;

interactions! {
  use stdlib;

  struct Fib(-U64,+U64);
  impl Fib(_, o) for U64(_, $n @ (0|1)){ U64(o, $n) }
  impl Fib(_, o) for U64(_, $n) {
    Add(f1,f2,o)
    Fib(n1,f1)
    Fib(n2,f2)
    U64(n1, $n-1)
    U64(n2, $n-2)
  }

  fn _main() {
    U64(n, $40)
    Fib(n, o)
    Print(o)
  }
}

fn main() {
  let mut stats = Stats::default();
  let mut net = BasicNet::new(LinkAlloc::new(ArrayBuffer::new(1 << 20)));
  _main().construct(&mut net, &Interactions);
  reduce_with_stats(&mut net, &Interactions, &mut stats);
  println!("{stats}");
}
