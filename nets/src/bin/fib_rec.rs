use internets_nets::*;

mod stdlib;

interactions! {
  type FibRec;

  use stdlib::Std;

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

  fn main(n){
    Fib(n, o)
    Print(o)
  }
}

fn main() {
  use stdlib::UseStd;
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
