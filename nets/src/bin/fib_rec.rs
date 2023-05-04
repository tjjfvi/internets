use internets_nets::*;

interactions! {
  type FibRec;

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

  impl Fib(_, o) for U64(_, $n @ (0 | 1)) {
    U64(o, $n)
  }
  impl Fib(_, o) for U64(_, $n) {
    U64(a, $(n-1))
    U64(b, $(n-2))
    Fib(a, x)
    Fib(b, y)
    Add(x, y, o)
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
  let n = args.get(1).map(|x| x.parse().unwrap()).unwrap_or(40);
  let mut stats = Stats::default();
  let mut net = BasicNet::new(LinkAlloc::new(ArrayBuffer::new(1 << 20)));
  let [a] = FibRec::U64(&mut net, n);
  let [b] = FibRec::main(&mut net);
  net.link(a, b);
  reduce_with_stats(&mut net, &FibRec, &mut stats);
  println!("{stats}");
}
