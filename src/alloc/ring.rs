use crate::*;

const MIN_DLL_LEN: Length = Length::of(3);

#[derive(Debug)]
pub struct RingAlloc<B: BufferMut> {
  buffer: B,
  alloc: Addr,
}

impl<B: BufferMut> DelegateBuffer for RingAlloc<B> {
  type Buffer = B;
  #[inline(always)]
  fn delegatee_buffer(&self) -> &Self::Buffer {
    &self.buffer
  }
}

impl<B: BufferMut> DelegateBufferMut for RingAlloc<B> {
  #[inline(always)]
  fn delegatee_buffer_mut(&mut self) -> &mut Self::Buffer {
    &mut self.buffer
  }
}

impl<B: BufferMut> Alloc for RingAlloc<B> {
  fn alloc(&mut self, data: &[Word]) -> Addr {
    let len = Length::of(data.len() as u32);
    let initial = self.alloc;
    loop {
      let addr = self.alloc;
      let mut free_len = self.word(addr).as_null_len();
      debug_assert!(free_len.non_zero());
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
      if free_len >= len {
        let remaining_len = free_len - len;
        let new_addr = addr + len;
        if remaining_len.non_zero() {
          *self.word_mut(new_addr) = Word::null_len(remaining_len);
        }
        if remaining_len >= MIN_DLL_LEN {
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
        fail!(panic!("OOM"));
      }
    }
  }

  fn free(&mut self, addr: Addr, len: Length) {
    debug_assert!(len.non_zero());
    self.assert_valid(addr, len);
    if cfg!(debug_assertions) {
      self.slice_mut(addr, len).fill(Word::NULL);
    }
    let next = self.alloc;
    let prev = next + self.word(next + Delta::of(1)).as_null_delta();
    *self.word_mut(addr) = Word::null_len(len);
    if len >= MIN_DLL_LEN {
      self.dll_link(prev, addr);
      self.dll_link(addr, next);
      self.alloc = addr;
    }
  }
}

impl<B: BufferMut> RingAlloc<B> {
  pub fn new(mut buffer: B) -> Self {
    safe! { assert!(buffer.len() > Length::of(0)) };
    let alloc_addr = buffer.origin();
    *buffer.word_mut(alloc_addr) = Word::null_len(buffer.len());
    RingAlloc {
      buffer,
      alloc: alloc_addr,
    }
  }

  fn dll_link(&mut self, a: Addr, b: Addr) {
    *self.word_mut(a + Delta::of(2)) = Word::null_delta(b - a);
    *self.word_mut(b + Delta::of(1)) = Word::null_delta(a - b);
  }

  fn dll_read_prev_next(&mut self, addr: Addr) -> (Addr, Addr) {
    (
      addr + self.word(addr + Delta::of(1)).as_null_delta(),
      addr + self.word(addr + Delta::of(2)).as_null_delta(),
    )
  }

  fn dll_try_read(&mut self, addr: Addr) -> Option<(Length, Option<(Addr, Addr)>)> {
    if (addr.0 as usize) >= self.buffer_bounds().end.0 as usize {
      return None;
    }
    let word = self.word(addr);
    if word.mode() != WordMode::Null {
      return None;
    }
    let len = word.as_null_len();
    debug_assert!(len > Length::of(0));
    Some((
      len,
      if len >= MIN_DLL_LEN {
        Some(self.dll_read_prev_next(addr))
      } else {
        None
      },
    ))
  }
}
