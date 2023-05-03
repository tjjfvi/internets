use internets_nets::*;

interactions! {
  type BubbleGen;

  struct False(+Bool);
  struct True(+Bool);

  struct U64(+U64, $u64);

  struct Add(-U64, -U64, +U64);
  struct Sub(-U64, -U64, +U64);
  struct Mul(-U64, -U64, +U64);
  struct Mod(-U64, -U64, +U64);
  struct Gt(-U64, -U64, +Bool);

  struct AddX(-U64, +U64, $u64);
  struct SubX(-U64, +U64, $u64);
  struct MulX(-U64, +U64, $u64);
  struct ModX(-U64, +U64, $u64);
  struct GtX(-U64, +Bool, $u64);

  struct Clone(-U64, +U64, +U64);
  struct Erase(-U64);

  impl Add(_, i, o) for U64(_, $n) { AddX(i, o, $n) }
  impl Sub(_, i, o) for U64(_, $n) { SubX(i, o, $n) }
  impl Mul(_, i, o) for U64(_, $n) { MulX(i, o, $n) }
  impl Mod(_, i, o) for U64(_, $n) { ModX(i, o, $n) }
  impl Gt(_, i, o) for U64(_, $n) { GtX(i, o, $n) }

  impl AddX(_, o, $x) for U64(_, $y) { U64(o, $(x.wrapping_add(y))) }
  impl SubX(_, o, $x) for U64(_, $y) { U64(o, $(x.wrapping_sub(y))) }
  impl MulX(_, o, $x) for U64(_, $y) { U64(o, $(x.wrapping_mul(y))) }
  impl ModX(_, o, $x) for U64(_, $y) { U64(o, $(y % x)) }
  impl GtX(_, o, $x) for U64(_, $y) if (x > y) { True(o) }
  impl GtX(_, o, $_) for U64(_, $_) { False(o) }

  impl Clone(_, o1, o2) for U64(_, $n) {
    U64(o1, $n)
    U64(o2, $n)
  }
  impl Erase(_) for U64(_, $_) {}

  struct Nil(+List);
  struct Cons(+List, -U64, -List);

  struct Sort(-List, +List);
  struct Insert(-List, -U64, +List);
  struct SwapIf(-Bool, -U64, -U64, -List, +List);

  struct Rnd(-U64, -U64, +List);
  struct Sum(-List, +U64);

  impl Sort(_, o) for Nil(_) {
    Nil(o)
  }
  impl Sort(_, o) for Cons(_, x, xs) {
    Sort(xs, s)
    Insert(s, x, o)
  }

  impl Insert(_, v, o) for Nil(_) {
    Cons(o, v, n)
    Nil(n)
  }
  impl Insert(_, v, o) for Cons(_, x, xs) {
    Clone(v, v0, v1)
    Clone(x, x0, x1)
    SwapIf(c, v0, x0, xs, o)
    Gt(x1, v1, c)
  }

  impl SwapIf(_, v, x, xs, o) for False(_) {
    Cons(o, v, a)
    Cons(a, x, xs)
  }
  impl SwapIf(_, v, x, xs, o) for True(_) {
    Cons(o, x, a)
    Insert(xs, v, a)
  }

  impl Rnd(_, s, o) for U64(_, $0) {
    Erase(s)
    Nil(o)
  }
  impl Rnd(_, s, o) for U64(_, $n) {
    Clone(s, s0, s1)
    MulX(s1, s2, $1664525)
    AddX(s2, s3, $1013904223)
    ModX(s3, s4, $4294967296)
    Cons(o, s0, o2)
    Rnd(n1, s4, o2)
    U64(n, $n)
    AddX(n, n1, $u64::MAX)
  }

  impl Sum(_, o) for Nil(_) {
    U64(o, $0)
  }
  impl Sum(_, o) for Cons(_, x, l) {
    Add(x, y, o)
    Sum(l, y)
  }

  struct Print(-U64);

  impl Print(_) for U64(_, $n) if { println!("{}", n); true } {}
  impl Print(_) for U64(_, $_) {}

  fn main() {
    U64(n, $5000)
    U64(s, $1)
    Rnd(n, s, l0)
    Sort(l0, l1)
    Sum(l1, o)
    Print(o)
  }
}

fn main() {
  let mut stats = Stats::default();
  let mut net = BasicNet::new(LinkAlloc::new(ArrayBuffer::new(1 << 20)));
  BubbleGen::main(&mut net);
  reduce_with_stats(&mut net, &BubbleGen, &mut stats);
  println!("{stats}");
}
