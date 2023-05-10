use internets_nets::*;

mod libs;

interactions! {
  use libs::nat;
  use libs::std;
  use libs::u64_nat;

  struct Fib(-Nat, +Nat);
  struct FibS(-Nat, +Nat);

  impl Fib(_, r) for Zero(_) { Zero(r) }
  impl Fib(_, r) for Succ(_, x) { FibS(x, r) }

  impl FibS(_, r) for Zero(_) { Succ(r, Zero(_)) }
  impl FibS(_, r) for Succ(_, nat::Clone(_, x0, x1)) {
    nat::Add(Fib(x0, _), FibS(x1, _), r)
  }

  fn _main(n: $u64){
    Print(NatToU64(Fib(U64ToNat(U64(_, $n), _), _), _))
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
