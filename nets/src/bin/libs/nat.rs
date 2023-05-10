use internets_nets::interactions;

interactions! {
  pub struct Zero(+Nat);
  pub struct Succ(+Nat, -Nat);

  pub struct Add(-Nat, -Nat, +Nat);
  pub struct Mul(-Nat, -Nat, +Nat);
  pub struct Exp(-Nat, -Nat, +Nat);

  pub struct Erase(-Nat);
  pub struct Clone(-Nat, +Nat, +Nat);

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
}
