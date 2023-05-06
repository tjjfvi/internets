use std::sync::atomic::Ordering;

use crate::*;

const MIN_DLL_LEN: Length = Length::of(3);

#[derive(Debug)]
pub struct RingAlloc<B: Buffer> {
  buffer: B,
  alloc: Addr,
}

impl<B: Buffer> DelegateBuffer for RingAlloc<B> {
  type Buffer<'a> = B where Self: 'a;
  #[inline(always)]
  fn buffer(&self) -> &Self::Buffer<'_> {
    &self.buffer
  }
}

impl<B: Buffer> Alloc for RingAlloc<B> {
  fn alloc(&mut self, len: Length) -> Addr {
    let initial = self.alloc;
    loop {
      let addr = self.alloc;
      let mut free_len = self.read_word(addr).as_null_len();
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
          self
            .word(new_addr)
            .write(Word::null_len(remaining_len), Ordering::Relaxed);
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
        return addr;
      }
      self.alloc = next;
      if self.alloc.0 == initial.0 {
        oom!();
      }
    }
  }

  fn free(&mut self, addr: Addr, len: Length) {
    debug_assert!(len.non_zero());
    self.assert_valid(addr, len);
    if cfg!(debug_assertions) {
      self.write_slice(addr, len, &vec![Word::NULL; len.length_words()][..])
    }
    let next = self.alloc;
    let prev = next + self.read_word(next + Delta::of(1)).as_null_delta();
    self
      .word(addr)
      .write(Word::null_len(len), Ordering::Relaxed);
    if len >= MIN_DLL_LEN {
      self.dll_link(prev, addr);
      self.dll_link(addr, next);
      self.alloc = addr;
    }
  }
}

impl<B: Buffer> RingAlloc<B> {
  pub fn new(buffer: B) -> Self {
    safe! { assert!(buffer.len() > Length::of(0)) };
    let alloc_addr = buffer.origin();
    buffer
      .word(alloc_addr)
      .write(Word::null_len(buffer.len()), Ordering::Relaxed);
    buffer
      .word(alloc_addr + Delta::of(1))
      .write(Word::NULL, Ordering::Relaxed);
    buffer
      .word(alloc_addr + Delta::of(2))
      .write(Word::NULL, Ordering::Relaxed);
    RingAlloc {
      buffer,
      alloc: alloc_addr,
    }
  }

  fn dll_link(&mut self, a: Addr, b: Addr) {
    self
      .word(a + Delta::of(2))
      .write(Word::null_delta(b - a), Ordering::Relaxed);
    self
      .word(b + Delta::of(1))
      .write(Word::null_delta(a - b), Ordering::Relaxed);
  }

  fn dll_read_prev_next(&mut self, addr: Addr) -> (Addr, Addr) {
    (
      addr + self.read_word(addr + Delta::of(1)).as_null_delta(),
      addr + self.read_word(addr + Delta::of(2)).as_null_delta(),
    )
  }

  fn dll_try_read(&mut self, addr: Addr) -> Option<(Length, Option<(Addr, Addr)>)> {
    if (addr.0 as usize) >= self.buffer_bounds().end.0 as usize {
      return None;
    }
    let word = self.read_word(addr);
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
