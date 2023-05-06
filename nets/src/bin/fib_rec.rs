use std::thread;

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
  let n = args.get(1).map(|x| x.parse().unwrap()).unwrap_or(40);
  let alloc = BumpAlloc::new(ArrayBuffer::new(1 << 28));
  let work = Pool::default();
  let mut net = BasicNet::new(&alloc, work.as_ref());
  let [a] = FibRec::U64(&mut net, n);
  let [b] = FibRec::main(&mut net);
  net.link(a, b);
  thread::scope(|s| {
    let threads = (0..8)
      .map(|_| {
        let mut net = std::mem::replace(&mut net, BasicNet::new(&alloc, work.as_ref()));
        s.spawn(move || {
          let mut stats = Stats::default();
          reduce_with_stats(&mut net, &FibRec, &mut stats);
          stats
        })
      })
      .collect::<Vec<_>>();
    drop(net);
    let stats = threads
      .into_iter()
      .map(|t| t.join().unwrap())
      .collect::<Vec<_>>();
    for (i, stats) in stats.iter().enumerate() {
      println!("{i}: {stats}");
    }
  });
}
