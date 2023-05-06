use internets_nets::*;

mod stdlib;

interactions! {
  type FibLinear;

  use stdlib::Std;

  struct Fib(-U64,+U64);
  impl Fib(_, o) for Std::U64(_, $0) { Std::U64(o, $0) }
  impl Fib(_, o) for Std::U64(_, $n) {
    Std::U64(i,$n-1)
    Std::U64(f0,$0)
    Std::U64(f1,$1)
    FibP(i, f1, f0, o )
  }

  struct FibP(-U64,-U64,-U64,+U64);
  impl FibP(_, o, a, o) for Std::U64(_, $0) { Std::Erase(a) }
  impl FibP(_, b, a, o) for Std::U64(_, $n) {
    Std::U64(i,$n-1)
    FibX( b, a, o, i)
  }

  struct FibX(-U64,-U64,+U64,-U64);
  impl FibX(_, a, o, i) for Std::U64(_, $b) {
    Std::U64(b,$b)
    Std::Clone(b,b1,b2)
    Std::Add(b1,a,c)
    FibP(i,c,b2,o)
  }

  fn main() {
    Std::U64(n, $1_000_000_000)
    Fib(n, o)
    Std::Print(o)
  }
}

fn main() {
  let mut stats = Stats::default();
  let mut net = BasicNet::new(LinkAlloc::new(ArrayBuffer::new(1 << 20)));
  FibLinear::main(&mut net);
  reduce_with_stats(&mut net, &FibLinear, &mut stats);
  println!("{stats}");
}
