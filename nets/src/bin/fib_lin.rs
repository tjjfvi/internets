use internets_nets::*;

mod stdlib;

interactions! {
  use stdlib;

  struct Fib {
    n: -U64,
    o: +U64,
  }

  impl Fib { n: _, o: U64(_, $0) } for U64(_, $0) {}
  impl Fib { n: _, o } for U64(_, $n) {
    FibX {
      n: U64(_, $n-1),
      x: U64(_, $0),
      y: U64(_, $1),
      o,
    }
  }

  struct FibX {
    n: -U64,
    x: -U64,
    y: -U64,
    o: +U64,
  }

  impl FibX {
    n: _,
    x: Erase(_),
    y: o,
    o,
  } for U64(_, $0) {}

  impl FibX {
    n: _,
    x,
    y: Clone(_, y0, y1),
    o,
  } for U64(_, $n) {
    FibX {
      n: U64(_, $n-1),
      x: y1,
      y: Add(x, y0, _),
      o,
    }
  }

  fn _main(n: $u64){
    Print(Fib { n: U64(_, $n), o: _ })
  }
}

fn main() {
  let args: Vec<_> = std::env::args().collect();
  let n = args.get(1).map(|x| x.parse().unwrap()).unwrap_or(1000000);
  let mut stats = Stats::default();
  let mut net = BasicNet::new(LinkAlloc::new(ArrayBuffer::new(1 << 28)));
  _main(n).construct(&mut net, &Interactions);
  reduce_with_stats(&mut net, &Interactions, &mut stats);
  println!("{stats}");
}
