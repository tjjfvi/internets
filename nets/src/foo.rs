use internets_interactions_macro::interactions;

interactions! {
  type Nat;

  struct Zero(+Nat);
  struct Succ(+Nat, -Nat);

  struct Add(-Nat, -Nat, +Nat);
  struct Mul(-Nat, -Nat, +Nat);
  struct Exp(-Nat, -Nat, +Nat);

  struct Erase(-Nat);
  struct Clone(-Nat);

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
  impl Add(_, a, b) for Succ(_, c) {
    Add(a, b, y)
    Succ(c, y)
  }

  impl Add(_, x, x) for Zero(_) {}
  impl Add(_, a, b) for Succ(_, c) {
    Add(a, b, y)
    Succ(c, y)
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

  fn main(n256) {
    Zero(n0)
    Succ(n1, n0)
    Succ(n2, n0)
    Clone(n2, n2_0, n2_1)
    Mul(n2_0, n2_1, n4)
    Clone(n4, n4_0, n4_1)
    Mul(n4_0, n4_1, n16)
    Clone(n16, n16_0, n16_1)
    Mul(n16_0, n16_1, n256)
  }
}
