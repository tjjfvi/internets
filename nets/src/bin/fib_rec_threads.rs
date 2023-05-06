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
  let threads = args.get(2).map(|x| x.parse().unwrap()).unwrap_or(4);
  let mut stats = vec![Stats::default(); threads];
  for _ in 0..10 {
    let buffer = ArrayBuffer::new(1 << 28);
    let alloc = SplitAlloc::new(&buffer, threads);
    let work = Steal::new(threads);
    let mut net = BasicNet::new(&alloc[0], &work[0]);
    let [a] = FibRec::U64(&mut net, n);
    let [b] = FibRec::main(&mut net);
    net.link(a, b);
    thread::scope(|s| {
      let threads = stats
        .iter_mut()
        .zip(work.into_iter())
        .into_iter()
        .enumerate()
        .map(|(i, (stats, work))| {
          let alloc = &alloc[i];
          Builder::new()
            .name(format!("worker_{}", i))
            .spawn_scoped(s, move || {
              let mut net = BasicNet::new(alloc, &work);
              reduce_with_stats(&mut net, &FibRec, stats);
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
  }
  for (i, stats) in stats.iter().enumerate() {
    println!("{i}: {stats}");
  }

  println!("{}", merge_stats(&stats[..]));
}
