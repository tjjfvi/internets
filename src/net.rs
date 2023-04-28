use crate::*;
use std::{
  fmt::Debug,
  ops::{Deref, DerefMut},
};

#[derive(Debug)]
pub struct Net<B: BufferMut> {
  pub(super) buffer: B,
  pub(super) alloc: Addr,
  pub(super) active: Vec<ActivePair>,
}

impl<B: BufferMut> Deref for Net<B> {
  type Target = B;

  fn deref(&self) -> &Self::Target {
    &self.buffer
  }
}

impl<B: BufferMut> DerefMut for Net<B> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.buffer
  }
}

#[derive(Debug)]
pub(super) struct ActivePair(pub(super) Word, pub(super) Word);
