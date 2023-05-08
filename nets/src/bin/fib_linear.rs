use internets_nets::*;

mod stdlib;

interactions! {
  use stdlib;

  struct Fib(-U64,+U64);
  impl Fib(_, o) for U64(_, $0) { U64(o, $0) }
  impl Fib(_, o) for U64(_, $n) {
    U64(i,$n-1)
    U64(f0,$0)
    U64(f1,$1)
    FibP(i, f1, f0, o )
  }

  struct FibP(-U64,-U64,-U64,+U64);
  impl FibP(_, o, a, o) for U64(_, $0) { Erase(a) }
  impl FibP(_, b, a, o) for U64(_, $n) {
    U64(i,$n-1)
    FibX( b, a, o, i)
  }

  struct FibX(-U64,-U64,+U64,-U64);
  impl FibX(_, a, o, i) for U64(_, $b) {
    U64(b,$b)
    Clone(b,b1,b2)
    Add(b1,a,c)
    FibP(i,c,b2,o)
  }

  fn _main() {
    U64(n, $1_000_000_000)
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
