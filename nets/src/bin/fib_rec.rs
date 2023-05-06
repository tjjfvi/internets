use internets_nets::*;

mod stdlib;

interactions! {
  type FibRec;

  use stdlib::Std;

  struct Fib(-U64, +U64);

  impl Fib(_, o) for Std::U64(_, $n @ (0 | 1)) {
    Std::U64(o, $n)
  }
  impl Fib(_, o) for Std::U64(_, $n) {
    Std::U64(a, $n-1)
    Std::U64(b, $n-2)
    Fib(a, x)
    Fib(b, y)
    Std::Add(x, y, o)
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
  for _ in 0..10 {
    let mut net = BasicNet::new(LinkAlloc::new(ArrayBuffer::new(1 << 28)), Vec::new());
    let [a] = FibRec::U64(&mut net, n);
    let [b] = FibRec::main(&mut net);
    net.link(a, b);
    reduce_with_stats(&mut net, &FibRec, &mut stats);
  }
  println!("{stats}");
}
