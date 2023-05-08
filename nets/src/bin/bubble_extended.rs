use internets_nets::*;

mod stdlib;

interactions! {
  use stdlib;

  struct Nil(+List);
  struct Cons(+List, -U64, -List);

  impl Erase(_) for Nil(_){}
  impl Erase(_) for Cons(_,head,rest){
    Erase(head)
    Erase(rest)
  }
  impl Clone(_, o1, o2 ) for Nil(_){
    Nil(o1)
    Nil(o2)
  }
  impl Clone(_, o1, o2 ) for Cons(_,head,rest){
    Clone(head,h1,h2)
    Clone(rest,r1,r2)
    Cons(o1,h1,r1)
    Cons(o2,h2,r2)
  }


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
  // impl Rnd(_, seed, list) for U64(_, $n) {
  //   U64(val, $n)
  //   U64(n, $n)
  //   AddX(n, m, $u64::MAX)
  //   Rnd(m, seed, rest)
  //   Cons(list, val, rest)
  // }

  impl Sum(_, o) for Nil(_) {
    U64(o, $0)
  }
  impl Sum(_, o) for Cons(_, x, l) {
    Add(x, y, o)
    Sum(l, y)
  }

  struct Check(-List, +List, +Bool);
  struct CheckX(-List, -U64, -List, +List, +Bool);
  struct CheckT(-Bool, -List, -U64, -List, +List, +Bool);

  impl Check(_, lo, c) for Nil(_){
    Nil(lo)
    True(c)
  }
  impl Check(_, out, result) for Cons(_, head, rest){
    Clone( head, _head, __head, )
    Clone( rest, _rest, __rest, )
    Cons(origList, _head, _rest)
    CheckX(__rest, __head, origList, out, result)
  }
  impl CheckX(_, cur, origList, out, result) for Cons(_, head, rest){
    Clone(head, _head, __head)
    Le( _head, cur, correct )
    CheckT( correct, rest, __head, origList, out, result)
  }
  impl CheckX(_, cur, origList, origList, result) for Nil(_){
    True(result)
    Erase(cur)
  }
  impl CheckT(_, rest, cur, origList, out, result) for True(_){
    CheckX( rest, cur, origList, out, result)
  }
  impl CheckT(_, rest, cur, origList, origList, result) for False(_){
    False( result )
    Erase( rest )
    Erase( cur )
  }

  struct Then(+next);
  struct Await(-next,-_a,+_a);
  impl Await(_, x, x ) for Then(_){}

  struct PrintThen(-U64,+next);
  impl PrintThen(_, next) for U64(_, $n) if { println!("{}", n); true } {
    Then(next)
  }
  impl PrintThen(_, next) for U64(_, $_) { Erase(next) }
  impl PrintThen(_, next) for Nil(_) { Then(next) }
  impl PrintThen(_, next) for Cons(_,head,rest) {
    PrintThen(head, then)
    Await(then, print, rest )
    PrintThen(print, next)
  }

  impl Print(_) for Nil(_) {}
  impl Print(_) for Cons(_, head, rest) {
    PrintThen(head, then)
    Await(then, print, rest )
    Print(print)
  }

  // fn main() {
  //   U64(n, $5000)
  //   U64(s, $1)
  //   Rnd(n, s, l0)
  //   Sort(l0, l1)
  //   Sum(l1, o)
  //   Print(o)
  // }

  fn _main() {
    U64(count, $5000)
    U64(seed, $1)
    Rnd(count, seed, random_list)


    // Clone( random_list, _random_list, __random_list )
    // PrintThen( _random_list, _next)
    // Erase( _next )
    // PrintThen( __random_list, __next)
    // Erase( __next )

    // Sort(random_list, sorted_list)
    // Clone( sorted_list, _sorted_list, __sorted_list )
    // PrintThen( _sorted_list, _next)
    // Await( _next, sum, __sorted_list )
    // Sum( sum, total)
    // Print( total )

    Sort(random_list, sorted_list)
    Check(sorted_list, checked_list, result)
    Print( result )
    Sum(checked_list, total)
    Print( total)
  }
}

fn main() {
  let mut stats = Stats::default();
  let mut net = BasicNet::new(LinkAlloc::new(ArrayBuffer::new(1 << 20)));
  _main().construct(&mut net, &Interactions);
  reduce_with_stats(&mut net, &Interactions, &mut stats);
  println!("{stats}");
}
