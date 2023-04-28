use crate::*;

const MIN_DLL_LEN: i32 = 12;

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
      debug_assert!(free_len.offset_bytes > 0);
      while let Some((len_inc, prev_next)) = self.dll_try_read(addr + free_len) {
        if prev_next.is_some() {
          break;
        }
        free_len = free_len + len_inc;
        if let Some((prev, next)) = prev_next {
          self.dll_link(prev, next);
        }
      }
      let (prev, next) = self.dll_read_prev_next(addr);
      if free_len.offset_bytes >= len.offset_bytes {
        let remaining_len = free_len - len;
        let new_addr = addr + len;
        if remaining_len.offset_bytes > 0 {
          *self.word_mut(new_addr) = Word::null(remaining_len);
        }
        if remaining_len.offset_bytes >= MIN_DLL_LEN {
          if prev.0 == addr.0 {
            self.dll_link(new_addr, new_addr);
          } else {
            self.dll_link(prev, new_addr);
            self.dll_link(new_addr, next);
          }
          self.alloc = new_addr;
        } else {
          self.dll_link(prev, next);
          self.alloc = next;
        }
        self.slice_mut(addr, len).copy_from_slice(data);
        return addr;
      }
      self.alloc = next;
      if self.alloc.0 == initial.0 {
        panic!("OOM");
      }
    }
  }

  pub fn free(&mut self, addr: Addr, len: Delta) {
    assert!(len.offset_bytes >= 4);
    self.assert_valid(addr, len.offset_bytes as usize);
    if cfg!(debug_assertions) {
      self.slice_mut(addr, len).fill(Word::NULL);
    }
    let next = self.alloc;
    let prev = next + self.word(next + Delta::of(1)).as_null();
    *self.word_mut(addr) = Word::null(len);
    if len.offset_bytes >= MIN_DLL_LEN {
      self.dll_link(prev, addr);
      self.dll_link(addr, next);
      self.alloc = addr;
    }
  }

  fn dll_link(&mut self, a: Addr, b: Addr) {
    *self.word_mut(a + Delta::of(2)) = Word::null(b - a);
    *self.word_mut(b + Delta::of(1)) = Word::null(a - b);
  }

  fn dll_read_prev_next(&mut self, addr: Addr) -> (Addr, Addr) {
    (
      addr + self.word(addr + Delta::of(1)).as_null(),
      addr + self.word(addr + Delta::of(2)).as_null(),
    )
  }

  fn dll_try_read(&mut self, addr: Addr) -> Option<(Delta, Option<(Addr, Addr)>)> {
    if (addr.0 as usize) >= self.buffer_bounds().end {
      return None;
    }
    let word = self.word(addr);
    if word.mode() != WordMode::Null {
      return None;
    }
    let len = word.as_null();
    debug_assert!(len.offset_bytes > 0);
    Some((
      len,
      if len.offset_bytes >= 12 {
        Some(self.dll_read_prev_next(addr))
      } else {
        None
      },
    ))
  }
}