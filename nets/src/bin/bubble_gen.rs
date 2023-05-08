use internets_nets::*;

mod stdlib;

interactions! {
  use stdlib;

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

  fn _main(n: $u64) {
    U64(n, $n)
    U64(s, $1)
    Rnd(n, s, l0)
    Sort(l0, l1)
    Sum(l1, o)
    Print(o)
  }
}

fn main() {
  let args: Vec<_> = std::env::args().collect();
  let n = args.get(1).map(|x| x.parse().unwrap()).unwrap_or(1000);
  let mut stats = Stats::default();
  let mut net = BasicNet::new(LinkAlloc::new(ArrayBuffer::new(1 << 20)));
  _main(n).construct(&mut net, &Interactions);
  reduce_with_stats(&mut net, &Interactions, &mut stats);
  println!("{stats}");
}
