use internets_nets::*;

mod stdlib;

interactions! {
  type BubbleExtended;

  use stdlib::Std;

  struct Nil(+List);
  struct Cons(+List, -U64, -List);

  impl Std::Erase(_) for Nil(_){}
  impl Std::Erase(_) for Cons(_,head,rest){
    Std::Erase(head)
    Std::Erase(rest)
  }
  impl Std::Clone(_, o1, o2 ) for Nil(_){
    Nil(o1)
    Nil(o2)
  }
  impl Std::Clone(_, o1, o2 ) for Cons(_,head,rest){
    Std::Clone(head,h1,h2)
    Std::Clone(rest,r1,r2)
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
  // impl Rnd(_, seed, list) for Std::U64(_, $n) {
  //   Std::U64(val, $n)
  //   Std::U64(n, $n)
  //   Std::AddX(n, m, $u64::MAX)
  //   Rnd(m, seed, rest)
  //   Cons(list, val, rest)
  // }

  impl Sum(_, o) for Nil(_) {
    Std::U64(o, $0)
  }
  impl Sum(_, o) for Cons(_, x, l) {
    Std::Add(x, y, o)
    Sum(l, y)
  }

  struct Check(-List, +List, +Bool);
  struct CheckX(-List, -U64, -List, +List, +Bool);
  struct CheckT(-Bool, -List, -U64, -List, +List, +Bool);

  impl Check(_, lo, c) for Nil(_){
    Nil(lo)
    Std::True(c)
  }
  impl Check(_, out, result) for Cons(_, head, rest){
    Std::Clone( head, _head, __head, )
    Std::Clone( rest, _rest, __rest, )
    Cons(origList, _head, _rest)
    CheckX(__rest, __head, origList, out, result)
  }
  impl CheckX(_, cur, origList, out, result) for Cons(_, head, rest){
    Std::Clone(head, _head, __head)
    Std::Le( _head, cur, correct )
    CheckT( correct, rest, __head, origList, out, result)
  }
  impl CheckX(_, cur, origList, origList, result) for Nil(_){
    Std::True(result)
    Std::Erase(cur)
  }
  impl CheckT(_, rest, cur, origList, out, result) for Std::True(_){
    CheckX( rest, cur, origList, out, result)
  }
  impl CheckT(_, rest, cur, origList, origList, result) for Std::False(_){
    Std::False( result )
    Std::Erase( rest )
    Std::Erase( cur )
  }

  struct Then(+next);
  struct Await(-next,-_a,+_a);
  impl Await(_, x, x ) for Then(_){}

  struct PrintThen(-U64,+next);
  impl PrintThen(_, next) for Std::U64(_, $n) if { println!("{}", n); true } {
    Then(next)
  }
  impl PrintThen(_, next) for Std::U64(_, $_) { Std::Erase(next) }
  impl PrintThen(_, next) for Nil(_) { Then(next) }
  impl PrintThen(_, next) for Cons(_,head,rest) {
    PrintThen(head, then)
    Await(then, print, rest )
    PrintThen(print, next)
  }

  impl Std::Print(_) for Nil(_) {}
  impl Std::Print(_) for Cons(_, head, rest) {
    PrintThen(head, then)
    Await(then, print, rest )
    Std::Print(print)
  }

  // fn main() {
  //   Std::U64(n, $5000)
  //   Std::U64(s, $1)
  //   Rnd(n, s, l0)
  //   Sort(l0, l1)
  //   Sum(l1, o)
  //   Std::Print(o)
  // }

  fn main() {
    Std::U64(count, $5000)
    Std::U64(seed, $1)
    Rnd(count, seed, randomList)


    // Std::Clone( randomList, _randomList, __randomList )
    // PrintThen( _randomList, _next)
    // Std::Erase( _next )
    // PrintThen( __randomList, __next)
    // Std::Erase( __next )

    // Sort(randomList, sortedList)
    // Std::Clone( sortedList, _sortedList, __sortedList )
    // PrintThen( _sortedList, _next)
    // Await( _next, sum, __sortedList )
    // Sum( sum, total)
    // Std::Print( total )

    Sort(randomList, sortedList)
    Check(sortedList, checkedList, result)
    Std::Print( result )
    Sum(checkedList, total)
    Std::Print( total)
  }
}

fn main() {
  let mut stats = Stats::default();
  let mut net = BasicNet::new(LinkAlloc::new(ArrayBuffer::new(1 << 20)));
  BubbleExtended::main(&mut net);
  reduce_with_stats(&mut net, &BubbleExtended, &mut stats);
  println!("{stats}");
}
