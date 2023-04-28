use crate::*;

impl Net {
  pub fn new(size: usize) -> Self {
    let mut buffer = vec![Word::NULL; size].into_boxed_slice();
    buffer[0] = Word::null(Delta::of(buffer.len() as i32));
    let alloc_addr = Addr(&buffer[0] as *const Word as *mut Word);
    Net {
      buffer,
      alloc: alloc_addr,
      active: Vec::new(),
    }
  }

  pub fn alloc(&mut self, data: &[Word]) -> Addr {
    let len = Delta::of(data.len() as i32);
    let initial = self.alloc;
    loop {
      let addr = self.alloc;
      let mut free_len = self.word(addr).as_null();
      while let Some((len_inc, prev_next)) = self.try_read_dll(addr + free_len) {
        if prev_next.is_some() {
          break;
        }
        free_len = free_len + len_inc;
        if let Some((prev, next)) = prev_next {
          self.link_dll(prev, next);
        }
      }
      let (_, prev_next) = self.read_dll(addr).unwrap();
      let (prev, next) = prev_next.unwrap();
      if free_len.offset_bytes >= len.offset_bytes {
        let remaining_len = free_len - len;
        if remaining_len.offset_bytes >= 12 {
          let new_addr = addr + len;
          if prev.0 == addr.0 {
            self.insert_dll(new_addr, remaining_len, new_addr, new_addr);
          } else {
            self.insert_dll(new_addr, remaining_len, prev, next);
          }
        } else {
          self.link_dll(prev, next);
          self.alloc = next;
        }
        unsafe { std::slice::from_raw_parts_mut(addr.0, len.offset_bytes as usize / 4) }
          .copy_from_slice(data);
        return addr;
      }
      self.alloc = next;
      if self.alloc.0 == initial.0 {
        panic!("OOM");
      }
    }
  }

  fn link_dll(&mut self, a: Addr, b: Addr) {
    *self.word_mut(a + Delta::of(2)) = Word::null(b - a);
    *self.word_mut(b + Delta::of(1)) = Word::null(a - b);
  }

  fn insert_dll(&mut self, addr: Addr, len: Delta, prev: Addr, next: Addr) {
    *self.word_mut(addr) = Word::null(len);
    if len.offset_bytes >= 12 {
      self.link_dll(prev, addr);
      self.link_dll(addr, next);
      self.alloc = addr;
    }
  }

  fn read_dll(&mut self, addr: Addr) -> Option<(Delta, Option<(Addr, Addr)>)> {
    let word = self.word(addr);
    if word.mode() != WordMode::Null {
      return None;
    }
    let len = word.as_null();
    Some((
      len,
      if len.offset_bytes >= 12 {
        Some((
          addr + self.word(addr + Delta::of(1)).as_null(),
          addr + self.word(addr + Delta::of(2)).as_null(),
        ))
      } else {
        None
      },
    ))
  }

  fn try_read_dll(&mut self, addr: Addr) -> Option<(Delta, Option<(Addr, Addr)>)> {
    if (addr.0 as usize) < self.buffer_bounds().end {
      self.read_dll(addr)
    } else {
      None
    }
  }

  pub fn free(&mut self, addr: Addr, len: Delta) {
    assert!(len.offset_bytes >= 4);
    self.assert_valid(addr, len.offset_bytes as usize);
    unsafe { std::slice::from_raw_parts_mut(addr.0, len.offset_bytes as usize / 4) }
      .fill(Word::NULL);
    let next = self.alloc;
    let prev = next + self.word(next + Delta::of(1)).as_null();
    self.insert_dll(addr, len, prev, next);
  }
}
