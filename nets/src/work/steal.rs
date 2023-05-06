use crate::*;
use cache_padded::CachePadded;
use std::{
  num::NonZeroU64,
  sync::{
    atomic::{AtomicIsize, AtomicU64, Ordering},
    Condvar, Mutex,
  },
};

#[derive(Debug)]
struct Stack {
  data: Box<[AtomicU64]>,
  top: CachePadded<AtomicIsize>,
  steal: CachePadded<AtomicIsize>,
}

impl Stack {
  pub fn pop(&self) -> Option<NonZeroU64> {
    loop {
      let idx = self.top.fetch_sub(1, Ordering::Relaxed) - 1;
      if idx < 0 {
        self.top.store(0, Ordering::Relaxed);
        return None;
      }
      let val = self.data[idx as usize].swap(0, Ordering::Relaxed);
      if val != 0 {
        return NonZeroU64::new(val);
      }
    }
  }
  pub fn push(&self, val: u64) -> usize {
    let idx = self.top.fetch_add(1, Ordering::Relaxed);
    self.data[idx as usize].store(val, Ordering::Relaxed);
    self.steal.store(idx, Ordering::Release);
    return idx as usize;
  }
  pub fn steal(&self) -> Option<NonZeroU64> {
    loop {
      let idx = self.steal.fetch_sub(1, Ordering::Relaxed) - 1;
      if idx < 0 {
        return None;
      }
      let val = self.data[idx as usize].swap(0, Ordering::Relaxed);
      if val != 0 {
        return NonZeroU64::new(val);
      }
    }
  }
}

#[derive(Debug)]
pub struct Steal {
  stacks: Vec<Stack>,
  waiting: Mutex<usize>,
  wake: Condvar,
}

impl Steal {
  pub fn new(stacks: usize, size: usize) -> Self {
    assert!(size.is_power_of_two());
    Steal {
      stacks: (0..stacks)
        .map(|_| Stack {
          data: unsafe { std::mem::transmute(vec![0u64; size].into_boxed_slice()) },
          top: CachePadded::new(AtomicIsize::new(0)),
          steal: CachePadded::new(AtomicIsize::new(0)),
        })
        .collect(),
      waiting: Mutex::new(0),
      wake: Condvar::new(),
    }
  }
  pub fn as_ref(&self, i: usize) -> StealRef {
    StealRef {
      i,
      steal: self,
      stack: &self.stacks[i],
    }
  }
}

#[derive(Debug)]
pub struct StealRef<'a> {
  i: usize,
  stack: &'a Stack,
  steal: &'a Steal,
}

impl<'a> Work for StealRef<'a> {
  fn add(&mut self, pair: ActivePair) {
    // println!("push {:?}", pair);
    let data = unsafe { std::mem::transmute::<ActivePair, u64>(pair) };
    self.stack.push(data);
    self.steal.wake.notify_one();
  }
  fn take(&mut self) -> Option<ActivePair> {
    loop {
      if let Some(x) = self
        .stack
        .pop()
        .or_else(|| {
          for (i, stack) in self.steal.stacks.iter().enumerate() {
            if i == self.i {
              continue;
            }
            if let Some(x) = stack.steal() {
              return Some(x);
            }
          }
          None
        })
        .map(|x| unsafe { std::mem::transmute(x) })
      {
        return Some(x);
      }
      let mut waiting = self.steal.waiting.lock().unwrap();
      *waiting += 1;
      if *waiting == self.steal.stacks.len() {
        self.steal.wake.notify_all();
        return None;
      } else {
        waiting = self.steal.wake.wait(waiting).unwrap();
        *waiting -= 1;
      }
    }
  }
}
