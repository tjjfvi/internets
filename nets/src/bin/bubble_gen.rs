use internets_nets::*;

mod stdlib;

interactions! {
  type BubbleGen;

  use stdlib::Std;

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
    Std::Clone(v, v0, v1)
    Std::Clone(x, x0, x1)
    SwapIf(c, v0, x0, xs, o)
    Std::Gt(x1, v1, c)
  }

  impl SwapIf(_, v, x, xs, o) for Std::False(_) {
    Cons(o, v, a)
    Cons(a, x, xs)
  }
  impl SwapIf(_, v, x, xs, o) for Std::True(_) {
    Cons(o, x, a)
    Insert(xs, v, a)
  }

  impl Rnd(_, s, o) for Std::U64(_, $0) {
    Std::Erase(s)
    Nil(o)
  }
  impl Rnd(_, s, o) for Std::U64(_, $n) {
    Std::Clone(s, s0, s1)
    Std::MulX(s1, s2, $1664525)
    Std::AddX(s2, s3, $1013904223)
    Std::ModX(s3, s4, $4294967296)
    Cons(o, s0, o2)
    Rnd(n1, s4, o2)
    Std::U64(n, $n)
    Std::AddX(n, n1, $u64::MAX)
  }

  impl Sum(_, o) for Nil(_) {
    Std::U64(o, $0)
  }
  impl Sum(_, o) for Cons(_, x, l) {
    Std::Add(x, y, o)
    Sum(l, y)
  }

  fn main() {
    Std::U64(n, $5000)
    Std::U64(s, $1)
    Rnd(n, s, l0)
    Sort(l0, l1)
    Sum(l1, o)
    Std::Print(o)
  }
}

fn main() {
  let mut stats = Stats::default();
  let mut net = BasicNet::new(LinkAlloc::new(ArrayBuffer::new(1 << 20)), Vec::new());
  BubbleGen::main(&mut net);
  reduce_with_stats(&mut net, &BubbleGen, &mut stats);
  println!("{stats}");
}
