use internets_nets::*;

mod libs;

interactions! {
  use libs::std;
  use libs::nat;
  use libs::u64_nat;

  struct Fib {
    n: -U64,
    o: +U64,
  }

  impl Fib { n: _, o: Zero(_) } for Zero(_) {}
  impl Fib { n: _, o } for Succ(_, n) {
    FibX {
      n,
      x: Zero(_),
      y: Succ(_, Zero(_)),
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
    x: nat::Erase(_),
    y: o,
    o,
  } for Zero(_) {}

  impl FibX {
    n: _,
    x,
    y: nat::Clone(_, y0, y1),
    o,
  } for Succ(_, n) {
    FibX {
      n,
      x: y1,
      y: nat::Add(x, y0, _),
      o,
    }
  }

  fn _main(n: $u64){
    Print(NatToU64(Fib { n: U64ToNat(U64(_, $n), _), o: _ }, _))
  }
}

fn main() {
  let args: Vec<_> = std::env::args().collect();
  let n = args.get(1).map(|x| x.parse().unwrap()).unwrap_or(32);
  let mut stats = Stats::default();
  let mut net = BasicNet::new(LinkAlloc::new(ArrayBuffer::new(1 << 28)));
  _main(n).construct(&mut net, &Interactions);
  reduce_with_stats(&mut net, &Interactions, &mut stats);
  println!("{stats}");
}
