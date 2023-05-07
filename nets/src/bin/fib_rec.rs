use internets_nets::*;

mod stdlib;

interactions! {
  use stdlib;

  struct Fib(-U64, +U64);

  impl Fib(_, o) for U64(_, $n @ (0 | 1)) {
    U64(o, $n)
  }
  impl Fib(_, o) for U64(_, $n) {
    U64(a, $n-1)
    U64(b, $n-2)
    Fib(a, x)
    Fib(b, y)
    Add(x, y, o)
  }

  fn _main(n: $u64){
    U64(n, $n)
    Fib(n, o)
    Print(o)
  }
}

fn main() {
  let args: Vec<_> = std::env::args().collect();
  let n = args.get(1).map(|x| x.parse().unwrap()).unwrap_or(64);
  let mut stats = Stats::default();
  let mut net = BasicNet::new(LinkAlloc::new(ArrayBuffer::new(1 << 28)));
  _main(n).construct(&mut net, &Interactions);
  reduce_with_stats(&mut net, &Interactions, &mut stats);
  println!("{stats}");
}
