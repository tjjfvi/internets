use internets_nets::*;

interactions! {
  type FibRecursive;

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
  impl Fib(_, o) for U64(_, $n @ (0|1)){ U64(o, $n) }
  impl Fib(_, o) for U64(_, $n) {
    Add(f1,f2,o)
    Fib(n1,f1)
    Fib(n2,f2)
    U64(n1, ${n-1})
    U64(n2, ${n-2})
  }

  fn main() {
    U64(n, $40)
    Fib(n, o)
    Print(o)
  }
}

fn main() {
  let mut stats = Stats::default();
  let mut net = Net::new(LinkAlloc::new(ArrayBuffer::new(1 << 20)));
  FibRecursive::main(&mut net);
  reduce_with_stats(&mut net, &FibRecursive, &mut stats);
  println!("{stats}");
}
