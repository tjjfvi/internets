use std::sync::{Arc, Condvar, Mutex};

use crate::*;
use crossbeam::deque::{Stealer, Worker};

#[derive(Debug)]
struct StealInner {
  stealers: Vec<Stealer<ActivePair>>,
  waiting: Mutex<usize>,
  wake: Condvar,
}

impl Steal {
  pub fn new(stacks: usize) -> Vec<Steal> {
    let workers = (0..stacks).map(|_| Worker::new_fifo()).collect::<Vec<_>>();
    let stealers = workers.iter().map(|w| w.stealer()).collect::<Vec<_>>();
    let inner = Arc::new(StealInner {
      stealers,
      waiting: Mutex::new(0),
      wake: Condvar::new(),
    });
    let steals = workers
      .into_iter()
      .map(|worker| Steal {
        worker,
        inner: inner.clone(),
      })
      .collect::<Vec<_>>();
    steals
  }
}

#[derive(Debug)]
pub struct Steal {
  worker: Worker<ActivePair>,
  inner: Arc<StealInner>,
}

impl<'a> Work for &'a Steal {
  #[inline(always)]
  fn add(&mut self, pair: ActivePair) {
    let was_empty = self.worker.is_empty();
    self.worker.push(pair);
    if was_empty {
      self.inner.wake.notify_one()
    }
  }
  fn take(&mut self) -> Option<ActivePair> {
    loop {
      if let Some(x) = self.worker.pop().or_else(|| {
        self
          .inner
          .stealers
          .iter()
          .map(|s| {
            std::iter::repeat_with(|| s.steal_batch_and_pop(&self.worker))
              .find(|x| !x.is_retry())
              .unwrap()
          })
          .find_map(|s| s.success())
      }) {
        return Some(x);
      }
      let mut waiting = self.inner.waiting.lock().unwrap();
      *waiting += 1;
      if *waiting == self.inner.stealers.len() {
        self.inner.wake.notify_all();
        return None;
      } else {
        waiting = self.inner.wake.wait(waiting).unwrap();
        if *waiting == self.inner.stealers.len() {
          return None;
        }
        *waiting -= 1;
      }
    }
  }
}
