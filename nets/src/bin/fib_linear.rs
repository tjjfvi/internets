use internets_nets::*;

interactions! {
  type FibLinear;

  struct U64(+U64, $u64);

  struct Add(-U64, -U64, +U64);
  struct AddX(-U64, +U64, $u64);

  struct Clone(-U64, +U64, +U64);
  struct Erase(-U64);
  struct Print(-U64);

  impl Add(_, i, o) for U64(_, $n) { AddX(i, o, $n) }
  impl AddX(_, o, $x) for U64(_, $y) { U64(o, $(x.wrapping_add(y))) }

  impl Clone(_, o1, o2) for U64(_, $n) {
    U64(o1, $n)
    U64(o2, $n)
  }
  impl Erase(_) for U64(_, $_) {}
  impl Print(_) for U64(_, $n) if { println!("{}", n); true } {}
  impl Print(_) for U64(_, $_) {}

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

  fn main() {
    U64(n, $1_000_000_000)
    Fib(n, o)
    Print(o)
  }
}

fn main() {
  let mut stats = Stats::default();
  let mut net = Net::new(LinkAlloc::new(ArrayBuffer::new(1 << 20)));
  FibLinear::main(&mut net);
  reduce_with_stats(&mut net, &FibLinear, &mut stats);
  println!("{stats}");
}
