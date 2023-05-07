use crate::*;

pub trait Marker {
  type Principal<'a>;
  type Auxiliary<'a>;
}

pub struct ConstructMarker;
impl Marker for ConstructMarker {
  type Principal<'a> = &'a mut LinkHalf;
  type Auxiliary<'a> = &'a mut LinkHalf;
}

pub struct DestructMarker;
impl Marker for DestructMarker {
  type Principal<'a> = ();
  type Auxiliary<'a> = LinkHalf;
}

pub struct GetKindMarker;
impl Marker for GetKindMarker {
  type Principal<'a> = ();
  type Auxiliary<'a> = ();
}

pub trait GetKind<I> {
  const KIND: Kind;
}

pub trait Construct<I> {
  fn construct<N: Net>(self, net: &mut N, interactions: &I);
}

pub trait Destruct {
  fn destruct<N: Net>(net: &mut N, addr: Addr) -> Self;
  fn free<N: Net>(net: &mut N, addr: Addr);
}
