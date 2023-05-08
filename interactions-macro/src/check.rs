use crate::*;

impl Program {
  pub fn check(&self) {
    self.ensure_unique(
      self
        .items
        .iter()
        .filter_map(Item::as_struct)
        .map(|x| &x.name),
    );

    for item in &self.items {
      match item {
        Item::Impl(i) => self.ensure_used_twice(i.all_idents()),
        Item::Fn(f) => self.ensure_used_twice(f.all_idents()),
        _ => {}
      }
    }
  }

  fn ensure_unique<'a, I: Iterator<Item = &'a Ident>>(&self, idents: I) {
    let mut seen = BTreeSet::new();
    for ident in idents {
      if !seen.insert(ident) {
        abort!(ident.span(), "duplicate identifier")
      }
    }
  }

  fn ensure_used_twice<'a, I: Iterator<Item = &'a Ident>>(&self, idents: I) {
    let mut all = BTreeSet::new();
    let mut once = BTreeSet::new();
    for ident in idents {
      if !all.insert(ident) {
        if !once.remove(ident) {
          emit_error!(ident.span(), "identifier used more than twice");
        }
      } else {
        once.insert(ident);
      }
    }
    for ident in once {
      emit_error!(ident.span(), "identifier used only once");
    }
  }
}
