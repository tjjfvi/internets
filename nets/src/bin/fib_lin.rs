use internets_nets::*;

interactions! {
  type FibLin;

  struct U64(+U64, $u64);

  struct Add(-U64, -U64, +U64);
  struct AddX(-U64, +U64, $u64);

  struct Clone(-U64, +U64, +U64);
  struct Erase(-U64);

  impl Add(_, i, o) for U64(_, $n) { AddX(i, o, $n) }
  impl AddX(_, o, $x) for U64(_, $y) { U64(o, $(x.wrapping_add(y))) }

  impl Clone(_, o1, o2) for U64(_, $n) {
    U64(o1, $n)
    U64(o2, $n)
  }
  impl Erase(_) for U64(_, $_) {}

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
    U64(n1, $(n-1))
  }

  struct Print(-U64);

  impl Print(_) for U64(_, $n) if { println!("{}", n); true } {}
  impl Print(_) for U64(_, $_) {}

  fn main(n){
    Fib(n, o)
    Print(o)
  }
}

fn main() {
  let args: Vec<_> = std::env::args().collect();
  let n = args.get(1).map(|x| x.parse().unwrap()).unwrap_or(64);
  let mut stats = Stats::default();
  let mut net = BasicNet::new(LinkAlloc::new(ArrayBuffer::new(1 << 28)));
  let [a] = FibLin::U64(&mut net, n);
  let [b] = FibLin::main(&mut net);
  net.link(a, b);
  reduce_with_stats(&mut net, &FibLin, &mut stats);
  println!("{stats}");
}
