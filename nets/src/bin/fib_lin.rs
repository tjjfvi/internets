use internets_nets::*;

mod stdlib;

interactions! {
  type FibLin;

  use stdlib::Std;

  struct Fib(-U64, +U64);

  impl Fib(_, o) for Std::U64(_, $0) {
    Std::U64(o, $0)
  }
  impl Fib(_, o) for Std::U64(_, $n) {
    FibX(n, x, y, o)
    Std::U64(n, $n-1)
    Std::U64(x, $0)
    Std::U64(y, $1)
  }

  struct FibX(-U64, -U64, -U64, +U64);

  impl FibX(_, x, y, y) for Std::U64(_, $0) {
    Std::Erase(x)
  }
  impl FibX(_, x, y, o) for Std::U64(_, $n) {
    Std::Clone(y, y0, y1)
    Std::Add(x, y0, z)
    FibX(n1, y1, z, o)
    Std::U64(n1, $n-1)
  }

  fn main(n){
    Fib(n, o)
    Std::Print(o)
  }
}

fn main() {
  use stdlib::UseStd;
  let args: Vec<_> = std::env::args().collect();
  let n = args.get(1).map(|x| x.parse().unwrap()).unwrap_or(64);
  let mut stats = Stats::default();
  let mut net = BasicNet::new(LinkAlloc::new(ArrayBuffer::new(1 << 28)), Vec::new());
  let [a] = FibLin::U64(&mut net, n);
  let [b] = FibLin::main(&mut net);
  net.link(a, b);
  reduce_with_stats(&mut net, &FibLin, &mut stats);
  println!("{stats}");
}
