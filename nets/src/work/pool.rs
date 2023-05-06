use crate::*;
use std::sync::{
  atomic::{AtomicU32, Ordering},
  Condvar, Mutex,
};

const CHUNK_SIZE: usize = 1024;
type Chunk = [ActivePair; CHUNK_SIZE];
const OVERFLOW_SIZE: usize = CHUNK_SIZE * 2;

#[derive(Debug, Default)]
pub struct Pool {
  chunks: Mutex<Vec<Chunk>>,
  available: Condvar,
  active: AtomicU32,
}

impl Pool {
  pub fn as_ref(&self) -> PoolRef<'_> {
    self.active.fetch_add(1, Ordering::Relaxed);
    PoolRef {
      pool: self,
      local: vec![],
    }
  }
}

#[derive(Debug)]
pub struct PoolRef<'a> {
  pool: &'a Pool,
  local: Vec<ActivePair>,
}

impl<'a> Work for PoolRef<'a> {
  fn add(&mut self, pair: ActivePair) {
    self.local.push(pair);
  }
  fn take(&mut self) -> Option<ActivePair> {
    if self.local.len() >= OVERFLOW_SIZE {
      let chunk: Chunk = unsafe { self.local[0..CHUNK_SIZE].try_into().unwrap_unchecked() };
      self.local.splice(0..CHUNK_SIZE, []);
      let mut chunks = self.pool.chunks.lock().unwrap();
      chunks.push(chunk);
      if chunks.len() == 1 {
        self.pool.available.notify_one();
      }
    }
    if let Some(w) = self.local.pop() {
      return Some(w);
    }
    let mut chunks = self.pool.chunks.lock().unwrap();
    if chunks.is_empty() {
      if self.pool.active.fetch_sub(1, Ordering::Relaxed) == 0 {
        self.pool.available.notify_all();
        return None;
      }
      loop {
        chunks = self.pool.available.wait(chunks).unwrap();
        if !chunks.is_empty() {
          break;
        }
        if self.pool.active.load(Ordering::Relaxed) == 0 {
          return None;
        }
      }
    }
    self.pool.active.fetch_add(1, Ordering::Relaxed);
    self.local.extend_from_slice(&chunks.pop().unwrap());
    self.local.pop()
  }
}
