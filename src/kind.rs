#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Kind {
  pub id: u32,
}

impl Kind {
  pub const fn of(id: u32) -> Kind {
    Kind { id }
  }
}
