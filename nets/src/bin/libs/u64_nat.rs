use internets_nets::interactions;

interactions! {
  use super::std;
  use super::nat;

  pub struct NatToU64(-Nat, +U64);
  struct _NatToU64(-Nat, +U64, $u64);

  impl NatToU64(_, U64(_, $0)) for Zero(_) {}
  impl NatToU64(_, o) for Succ(_, x) {
    _NatToU64(x, o, $1)
  }

  impl _NatToU64(_, U64(_, $n), $n) for Zero(_) {}
  impl _NatToU64(_, o, $n) for Succ(_, x) {
    _NatToU64(x, o, $n+1)
  }

  pub struct U64ToNat(-U64, +Nat);

  impl U64ToNat(_, Zero(_)) for U64(_, $0) {}
  impl U64ToNat(_, Succ(_, U64ToNat(U64(_, $n-1), _))) for U64(_, $n) {}
}
