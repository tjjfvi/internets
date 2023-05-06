use std::thread::{self, Builder};

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
  let buffer = ArrayBuffer::new(1 << 28);
  let threads = 8;
  let alloc = SplitAlloc::new(&buffer, threads);
  let work = Steal::new(threads, 1 << 20);
  let mut net = BasicNet::new(&alloc[0], work.as_ref(0));
  let [a] = FibRec::U64(&mut net, n);
  let [b] = FibRec::main(&mut net);
  net.link(a, b);
  let stats = thread::scope(|s| {
    let threads = (0..threads)
      .map(|i| {
        let mut net = BasicNet::new(&alloc[i], work.as_ref(i));
        Builder::new()
          .name(format!("worker_{}", i))
          .spawn_scoped(s, move || {
            let mut stats = Stats::default();
            reduce_with_stats(&mut net, &FibRec, &mut stats);
            stats
          })
          .unwrap()
      })
      .collect::<Vec<_>>();
    let stats = threads
      .into_iter()
      .map(|t| t.join().unwrap())
      .collect::<Vec<_>>();
    stats
  });
  // dbg!(work);
  for (i, stats) in stats.iter().enumerate() {
    println!("{i}: {stats}");
  }
}
