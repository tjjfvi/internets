use crate::*;

pub enum LinkHalf {
  From(Addr),
  Kind(Kind),
  Port(Addr, PortMode),
}

impl Net {
  #[inline(always)]
  pub fn link(&mut self, a: LinkHalf, b: LinkHalf) {
    let a = self.get_link_half(a);
    let b = self.get_link_half(b);
    use LinkHalf::*;
    use PortMode::*;
    match (a, b) {
      (Port(a, Auxiliary), Port(b, Auxiliary)) => self.link_aux_aux(a, b),
      (Port(a, Auxiliary), Port(b, Principal)) => self.link_aux_prn(a, b),
      (Port(a, Auxiliary), Kind(b)) => self.link_aux_nil(a, b),
      (Port(a, Principal), Port(b, Auxiliary)) => self.link_aux_prn(b, a),
      (Port(a, Principal), Port(b, Principal)) => self.link_prn_prn(a, b),
      (Port(a, Principal), Kind(b)) => self.link_prn_nil(a, b),
      (Kind(a), Port(b, Auxiliary)) => self.link_aux_nil(b, a),
      (Kind(a), Port(b, Principal)) => self.link_prn_nil(b, a),
      (Kind(a), Kind(b)) => self.link_nil_nil(a, b),
      _ => unreachable!(),
    }
  }

  #[inline(always)]
  fn get_link_half(&self, link_half: LinkHalf) -> LinkHalf {
    match link_half {
      LinkHalf::From(addr) => {
        let word = self.word(addr);
        match word.mode() {
          WordMode::Kind => LinkHalf::Kind(word.as_kind()),
          WordMode::Port(mode) => LinkHalf::Port(addr + word.as_port(), mode),
          _ => unreachable!(),
        }
      }
      x => x,
    }
  }

  fn link_aux_aux(&mut self, a: Addr, b: Addr) {
    *self.word_mut(a) = Word::port(b - a, PortMode::Auxiliary);
    *self.word_mut(b) = Word::port(a - b, PortMode::Auxiliary);
  }

  fn link_aux_prn(&mut self, a: Addr, b: Addr) {
    *self.word_mut(a) = Word::port(b - a, PortMode::Principal);
  }

  fn link_aux_nil(&mut self, a: Addr, b: Kind) {
    *self.word_mut(a) = Word::kind(b)
  }

  fn link_prn_prn(&mut self, a: Addr, b: Addr) {
    self.active.push(ActivePair(
      Word::port(a - self.origin(), PortMode::Principal),
      Word::port(b - self.origin(), PortMode::Principal),
    ));
  }

  fn link_prn_nil(&mut self, a: Addr, b: Kind) {
    self.active.push(ActivePair(
      Word::port(a - self.origin(), PortMode::Principal),
      Word::kind(b),
    ));
  }

  fn link_nil_nil(&mut self, _a: Kind, _b: Kind) {
    // they just annihilate
  }
}
