use internets_nets::*;

mod stdlib;

interactions! {
  use stdlib;

  struct Fib(-U64, +U64);

  impl Fib(_, o) for U64(_, $0) {
    U64(o, $0)
  }
  impl Fib(_, o) for U64(_, $n) {
    FibX(n, x, y, o)
    U64(n, $n-1)
    U64(x, $0)
    U64(y, $1)
  }

  struct FibX(-U64, -U64, -U64, +U64);

  impl FibX(_, x, y, y) for U64(_, $0) {
    Erase(x)
  }
  impl FibX(_, x, y, o) for U64(_, $n) {
    Clone(y, y0, y1)
    Add(x, y0, z)
    FibX(n1, y1, z, o)
    U64(n1, $n-1)
  }

  fn _main(n: $u64){
    U64(n, $n)
    Fib(n, o)
    Print(o)
  }
}

fn main() {
  let args: Vec<_> = std::env::args().collect();
  let n = args.get(1).map(|x| x.parse().unwrap()).unwrap_or(1000000);
  let mut stats = Stats::default();
  let mut net = BasicNet::new(LinkAlloc::new(ArrayBuffer::new(1 << 28)));
  _main(n).construct(&mut net, &Interactions);
  reduce_with_stats(&mut net, &Interactions, &mut stats);
  println!("{stats}");
}
