use internets_nets::interactions;

interactions! {
  type NatNum;

  struct Zero(+Nat);
  struct Succ(+Nat, -Nat);

  struct Add(-Nat, -Nat, +Nat);
  struct Mul(-Nat, -Nat, +Nat);
  struct Exp(-Nat, -Nat, +Nat);

  struct Erase(-Nat);
  struct Clone(-Nat, +Nat, +Nat);

  fn foo(x, y) {
    Clone(x, y, z)
    Erase(z)
  }

  impl Erase(_) for Zero(_) {}
  impl Erase(_) for Succ(_, x) {
    Erase(x)
  }

  impl Clone(_, a, b) for Zero(_) {
    Zero(a)
    Zero(b)
  }
  impl Clone(_, a, b) for Succ(_, x) {
    Clone(x, y, z)
    Succ(a, y)
    Succ(b, z)
  }

  impl Add(_, x, x) for Zero(_) {}
  impl Add(_, y, o) for Succ(_, x) {
    Add(x, y, o2)
    Succ(o, o2)
  }

  impl Mul(_, x, o) for Zero(_) {
    Erase(x)
    Zero(o)
  }
  impl Mul(_, x, o) for Succ(_, y) {
    Clone(x, x1, x2)
    Add(x2, o1, o)
    Mul(y, x1, o1)
  }

  impl Exp(_, x, o) for Zero(_) {
    Erase(x)
    Succ(o, z)
    Zero(z)
  }
  impl Exp(_, x, o) for Succ(_, y) {
    Clone(x, x1, x2)
    Mul(x2, o1, o)
    Exp(y, x1, o1)
  }

  fn square(i, o) {
    Clone(i, i0, i1)
    Mul(i0, i1, o)
  }

  fn main(n65536) {
    Zero(n0)
    Succ(n1, n0)
    Succ(n2, n1)
    square(n2, n4)
    square(n4, n16)
    square(n16, n256)
    square(n256, n65536)
  }
}

fn main() {
  use internets_nets::*;
  let mut stats = Stats::default();
  let buffer = ArrayBuffer::new(1 << 18);
  for _ in 0..1000 {
    let mut net = BasicNet::new(LinkAlloc::new(buffer.as_ref()), Vec::new());
    let free = net.alloc_write(&[Word::NULL]);
    let [free_0] = NatNum::main(&mut net);
    net.link(free_0, LinkHalf::Port(free, PortMode::Auxiliary));
    reduce_with_stats(&mut net, &NatNum, &mut stats);
  }
  eprintln!("{stats}");
}
