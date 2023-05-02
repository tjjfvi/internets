use internets_nets::*;

/*
// Sorts a list

// sort : List -> List
(Sort Nil)         = Nil
(Sort (Cons x xs)) = (Insert x (Sort xs))

// Insert : U60 -> List -> List
(Insert v Nil)         = (Cons v Nil)
(Insert v (Cons x xs)) = (SwapGT (> v x) v x xs)

// SwapGT : U60 -> U60 -> U60 -> List -> List
(SwapGT 0 v x xs) = (Cons v (Cons x xs))
(SwapGT 1 v x xs) = (StrictCons x (Insert v xs))
  (StrictCons e !t) = (Cons e t)

// Generates a random list
(Rnd 0 s) = (Nil)
(Rnd n s) = (Cons s (Rnd (- n 1) (% (+ (* s 1664525) 1013904223) 4294967296)))

// Sums a list
(Sum Nil)         = 0
(Sum (Cons x xs)) = (+ x (Sum xs))

(Main n) = (Sum (Sort (Rnd n 1)))
*/

#[derive(Clone, Copy, Debug)]
pub struct Bubble;

impl Bubble {
  pub const FALSE: Kind = Kind::of(0);
  pub const TRUE: Kind = Kind::of(1);

  pub const U64: Kind = Kind::of(2);

  pub const ADD: Kind = Kind::of(3);
  pub const SUB: Kind = Kind::of(4);
  pub const MUL: Kind = Kind::of(5);
  pub const MOD: Kind = Kind::of(6);
  pub const GT: Kind = Kind::of(7);

  pub const ADD_: Kind = Kind::of(8);
  pub const SUB_: Kind = Kind::of(9);
  pub const MUL_: Kind = Kind::of(10);
  pub const MOD_: Kind = Kind::of(11);
  pub const GT_: Kind = Kind::of(12);

  pub const CLONE: Kind = Kind::of(13);
  pub const ERASE: Kind = Kind::of(14);

  pub const NIL: Kind = Kind::of(17);
  pub const CONS: Kind = Kind::of(18);

  pub const SORT: Kind = Kind::of(19);
  pub const INSERT: Kind = Kind::of(20);
  pub const SWAP_IF: Kind = Kind::of(21);

  pub const RND: Kind = Kind::of(22);
  pub const SUM: Kind = Kind::of(23);
}

impl Interactions for Bubble {
  #[inline(always)]
  fn reduce<M: Alloc>(
    &self,
    net: &mut Net<M>,
    (a_kind, a_addr): (Kind, Addr),
    (b_kind, b_addr): (Kind, Addr),
  ) {
    macro_rules! partial_op {
      ($new_kind:expr) => {{
        const CHUNK: &'static [Word] = &[Word::kind($new_kind), Word::NULL, Word::NULL, Word::NULL];
        let chunk = net.alloc_write(CHUNK);
        net.write_payload(
          chunk + Delta::of(1),
          net.read_payload::<u64>(a_addr + Delta::of(1)),
        );
        net.link(
          LinkHalf::From(b_addr + Delta::of(1)),
          LinkHalf::Port(chunk, PortMode::Principal),
        );
        net.link(
          LinkHalf::From(b_addr + Delta::of(2)),
          LinkHalf::Port(chunk + Delta::of(3), PortMode::Auxiliary),
        );
        net.free(a_addr, Length::of(3));
        net.free(b_addr, Length::of(3));
      }};
    }
    macro_rules! finish_op {
      ($op:expr) => {{
        let x_addr = a_addr + Delta::of(1);
        let y_addr = b_addr + Delta::of(1);
        net.write_payload(
          x_addr,
          ($op)(
            net.read_payload::<u64>(x_addr),
            net.read_payload::<u64>(y_addr),
          ),
        );
        net.link(
          LinkHalf::Port(a_addr, PortMode::Principal),
          LinkHalf::From(b_addr + Delta::of(3)),
        );
        net.free(b_addr, Length::of(4));
      }};
    }
    macro_rules! finish_cmp {
      ($cmp:expr) => {{
        let x_addr = a_addr + Delta::of(1);
        let y_addr = b_addr + Delta::of(1);
        let kind = if ($cmp)(
          net.read_payload::<u64>(x_addr),
          net.read_payload::<u64>(y_addr),
        ) {
          Bubble::TRUE
        } else {
          Bubble::FALSE
        };
        net.link(LinkHalf::Kind(kind), LinkHalf::From(b_addr + Delta::of(3)));
        net.free(a_addr, Length::of(3));
        net.free(b_addr, Length::of(4));
      }};
    }
    match (a_kind, b_kind) {
      (Bubble::U64, Bubble::ADD) => partial_op!(Bubble::ADD_),
      (Bubble::U64, Bubble::SUB) => partial_op!(Bubble::SUB_),
      (Bubble::U64, Bubble::MUL) => partial_op!(Bubble::MUL_),
      (Bubble::U64, Bubble::MOD) => partial_op!(Bubble::MOD_),
      (Bubble::U64, Bubble::GT) => partial_op!(Bubble::GT_),
      (Bubble::U64, Bubble::ADD_) => finish_op!(|x: u64, y| x.wrapping_add(y)),
      (Bubble::U64, Bubble::SUB_) => finish_op!(|x: u64, y| x.wrapping_sub(y)),
      (Bubble::U64, Bubble::MUL_) => finish_op!(|x: u64, y| x.wrapping_mul(y)),
      (Bubble::U64, Bubble::MOD_) => finish_op!(|x, y| x % y),
      (Bubble::U64, Bubble::GT_) => finish_cmp!(|x, y| x > y),
      (Bubble::U64, Bubble::CLONE) => {
        const CHUNK: &'static [Word] = &[Word::kind(Bubble::U64), Word::NULL, Word::NULL];
        let chunk = net.alloc_write(CHUNK);
        net.write_payload::<u64>(
          chunk + Delta::of(1),
          net.read_payload(a_addr + Delta::of(1)),
        );
        net.link(
          LinkHalf::From(b_addr + Delta::of(1)),
          LinkHalf::Port(a_addr, PortMode::Principal),
        );
        net.link(
          LinkHalf::From(b_addr + Delta::of(2)),
          LinkHalf::Port(chunk, PortMode::Principal),
        );
        net.free(b_addr, Length::of(3));
      }
      (Bubble::U64, Bubble::ERASE) => {
        net.free(a_addr, Length::of(3));
      }
      (Bubble::NIL, Bubble::SORT) => {
        net.link(
          LinkHalf::From(b_addr + Delta::of(1)),
          LinkHalf::Kind(Bubble::NIL),
        );
        net.free(b_addr, Length::of(2));
      }
      (Bubble::CONS, Bubble::SORT) => {
        const CHUNK: &'static [Word] = &[Word::kind(Bubble::INSERT), Word::NULL, Word::NULL];
        let chunk = net.alloc_write(CHUNK);
        net.link(
          LinkHalf::From(b_addr + Delta::of(1)),
          LinkHalf::Port(chunk + Delta::of(2), PortMode::Auxiliary),
        );
        net.link(
          LinkHalf::From(a_addr + Delta::of(1)),
          LinkHalf::Port(chunk + Delta::of(1), PortMode::Auxiliary),
        );
        net.link(
          LinkHalf::From(a_addr + Delta::of(2)),
          LinkHalf::Port(b_addr, PortMode::Principal),
        );
        net.link(
          LinkHalf::Port(b_addr + Delta::of(1), PortMode::Auxiliary),
          LinkHalf::Port(chunk, PortMode::Principal),
        );
        net.free(a_addr, Length::of(3));
      }
      (Bubble::NIL, Bubble::INSERT) => {
        const CHUNK: &'static [Word] = &[
          Word::kind(Bubble::CONS),
          Word::NULL,
          Word::kind(Bubble::NIL),
        ];
        let chunk = net.alloc_write(CHUNK);
        net.link(
          LinkHalf::From(b_addr + Delta::of(1)),
          LinkHalf::Port(chunk + Delta::of(1), PortMode::Auxiliary),
        );
        net.link(
          LinkHalf::From(b_addr + Delta::of(2)),
          LinkHalf::Port(chunk, PortMode::Principal),
        );
        net.free(b_addr, Length::of(3));
      }
      (Bubble::CONS, Bubble::INSERT) => {
        const CHUNK: &'static [Word] = &[
          Word::kind(Bubble::CLONE),
          Word::port(Delta::of(5), PortMode::Principal),
          Word::port(Delta::of(8), PortMode::Auxiliary),
          Word::kind(Bubble::CLONE),
          Word::port(Delta::of(3), PortMode::Auxiliary),
          Word::port(Delta::of(6), PortMode::Auxiliary),
          Word::kind(Bubble::GT),
          Word::port(Delta::of(-3), PortMode::Auxiliary),
          Word::port(Delta::of(1), PortMode::Principal),
          Word::kind(Bubble::SWAP_IF),
          Word::port(Delta::of(-8), PortMode::Auxiliary),
          Word::port(Delta::of(-6), PortMode::Auxiliary),
          Word::NULL,
          Word::NULL,
        ];
        let chunk = net.alloc_write(CHUNK);
        net.link(
          LinkHalf::From(a_addr + Delta::of(1)),
          LinkHalf::Port(chunk + Delta::of(3), PortMode::Principal),
        );
        net.link(
          LinkHalf::From(b_addr + Delta::of(1)),
          LinkHalf::Port(chunk, PortMode::Principal),
        );
        net.link(
          LinkHalf::From(a_addr + Delta::of(2)),
          LinkHalf::Port(chunk + Delta::of(12), PortMode::Auxiliary),
        );
        net.link(
          LinkHalf::From(b_addr + Delta::of(2)),
          LinkHalf::Port(chunk + Delta::of(13), PortMode::Auxiliary),
        );
        net.free(a_addr, Length::of(3));
        net.free(b_addr, Length::of(3));
      }
      (Bubble::FALSE, Bubble::SWAP_IF) => {
        const CHUNK: &'static [Word] = &[
          Word::kind(Bubble::CONS),
          Word::NULL,
          Word::port(Delta::of(1), PortMode::Principal),
          Word::kind(Bubble::CONS),
          Word::NULL,
          Word::NULL,
        ];
        let chunk = net.alloc_write(CHUNK);
        net.link(
          LinkHalf::From(b_addr + Delta::of(1)),
          LinkHalf::Port(chunk + Delta::of(1), PortMode::Auxiliary),
        );
        net.link(
          LinkHalf::From(b_addr + Delta::of(2)),
          LinkHalf::Port(chunk + Delta::of(4), PortMode::Auxiliary),
        );
        net.link(
          LinkHalf::From(b_addr + Delta::of(3)),
          LinkHalf::Port(chunk + Delta::of(5), PortMode::Auxiliary),
        );
        net.link(
          LinkHalf::From(b_addr + Delta::of(4)),
          LinkHalf::Port(chunk, PortMode::Principal),
        );
        net.free(b_addr, Length::of(5))
      }
      (Bubble::TRUE, Bubble::SWAP_IF) => {
        const CHUNK: &'static [Word] = &[
          Word::kind(Bubble::CONS),
          Word::NULL,
          Word::port(Delta::of(3), PortMode::Auxiliary),
          Word::kind(Bubble::INSERT),
          Word::NULL,
          Word::NULL,
        ];
        let chunk = net.alloc_write(CHUNK);
        net.link(
          LinkHalf::From(b_addr + Delta::of(2)),
          LinkHalf::Port(chunk + Delta::of(1), PortMode::Auxiliary),
        );
        net.link(
          LinkHalf::From(b_addr + Delta::of(1)),
          LinkHalf::Port(chunk + Delta::of(4), PortMode::Auxiliary),
        );
        net.link(
          LinkHalf::From(b_addr + Delta::of(3)),
          LinkHalf::Port(chunk + Delta::of(3), PortMode::Principal),
        );
        net.link(
          LinkHalf::From(b_addr + Delta::of(4)),
          LinkHalf::Port(chunk, PortMode::Principal),
        );
        net.free(b_addr, Length::of(5))
      }
      (Bubble::U64, Bubble::RND) => {
        let value = net.read_payload::<u64>(a_addr + Delta::of(1));
        if value == 0 {
          net.link(
            LinkHalf::From(b_addr + Delta::of(1)),
            LinkHalf::Kind(Bubble::ERASE),
          );
          net.link(
            LinkHalf::From(b_addr + Delta::of(2)),
            LinkHalf::Kind(Bubble::NIL),
          );
          net.free(a_addr, Length::of(3));
          net.free(b_addr, Length::of(3));
        } else {
          const CHUNK: &'static [Word] = &[
            Word::kind(Bubble::RND),
            Word::port(Delta::of(23), PortMode::Auxiliary),
            Word::port(Delta::of(6), PortMode::Auxiliary),
            Word::kind(Bubble::CLONE), // !
            Word::port(Delta::of(3), PortMode::Auxiliary),
            Word::port(Delta::of(8), PortMode::Principal),
            Word::kind(Bubble::CONS), // !
            Word::port(Delta::of(-3), PortMode::Auxiliary),
            Word::port(Delta::of(-6), PortMode::Auxiliary),
            Word::kind(Bubble::ADD_), // !
            u64_0!(-1_i64),
            u64_1!(-1_i64),
            Word::port(Delta::of(-12), PortMode::Principal),
            Word::kind(Bubble::MUL_),
            u64_0!(1664525),
            u64_1!(1664525),
            Word::port(Delta::of(1), PortMode::Principal),
            Word::kind(Bubble::ADD_),
            u64_0!(1013904223),
            u64_1!(1013904223),
            Word::port(Delta::of(1), PortMode::Principal),
            Word::kind(Bubble::MOD_),
            u64_0!(4294967296),
            u64_1!(4294967296),
            Word::port(Delta::of(-23), PortMode::Auxiliary),
          ];
          let chunk = net.alloc_write(CHUNK);
          net.link(
            LinkHalf::Port(a_addr, PortMode::Principal),
            LinkHalf::Port(chunk + Delta::of(9), PortMode::Principal),
          );
          net.link(
            LinkHalf::From(b_addr + Delta::of(1)),
            LinkHalf::Port(chunk + Delta::of(3), PortMode::Principal),
          );
          net.link(
            LinkHalf::From(b_addr + Delta::of(2)),
            LinkHalf::Port(chunk + Delta::of(6), PortMode::Principal),
          );
          net.free(b_addr, Length::of(3));
        }
      }
      (Bubble::NIL, Bubble::SUM) => {
        const CHUNK: &'static [Word] = &[Word::kind(Bubble::U64), u64_0!(0), u64_0!(0)];
        let chunk = net.alloc_write(CHUNK);
        net.link(
          LinkHalf::From(b_addr + Delta::of(1)),
          LinkHalf::Port(chunk, PortMode::Principal),
        );
        net.free(b_addr, Length::of(2));
      }
      (Bubble::CONS, Bubble::SUM) => {
        const CHUNK: &'static [Word] = &[Word::kind(Bubble::ADD), Word::NULL, Word::NULL];
        let chunk = net.alloc_write(CHUNK);
        net.link(
          LinkHalf::From(a_addr + Delta::of(1)),
          LinkHalf::Port(chunk, PortMode::Principal),
        );
        net.link(
          LinkHalf::From(b_addr + Delta::of(1)),
          LinkHalf::Port(chunk + Delta::of(2), PortMode::Auxiliary),
        );
        net.link(
          LinkHalf::Port(chunk + Delta::of(1), PortMode::Auxiliary),
          LinkHalf::Port(b_addr + Delta::of(1), PortMode::Auxiliary),
        );
        net.link(
          LinkHalf::From(a_addr + Delta::of(2)),
          LinkHalf::Port(b_addr, PortMode::Principal),
        );
        net.free(a_addr, Length::of(3));
      }

      _ => fail!(unimplemented!("{:?} {:?}", a_kind, b_kind)),
    };
  }
}

fn main() {
  let mut net = Net::new(RingAlloc::new(ArrayBuffer::new(1 << 29)));
  let base = net.alloc_write(&[
    Word::port(Delta::of(2), PortMode::Auxiliary),
    // Word::port(Delta::of(7), PortMode::Auxiliary),
    Word::kind(Bubble::SUM),
    Word::port(Delta::of(-2), PortMode::Auxiliary),
    Word::kind(Bubble::SORT),
    Word::port(Delta::of(-3), PortMode::Principal),
    Word::kind(Bubble::RND),
    Word::port(Delta::of(5), PortMode::Principal),
    // Word::port(Delta::of(-7), PortMode::Auxiliary),
    Word::port(Delta::of(-4), PortMode::Principal),
    Word::kind(Bubble::U64),
    u64_0!(5000),
    u64_1!(5000),
    Word::kind(Bubble::U64),
    u64_0!(1),
    u64_1!(1),
  ]);
  net.link(
    LinkHalf::Port(base + Delta::of(5), PortMode::Principal),
    LinkHalf::Port(base + Delta::of(8), PortMode::Principal),
  );
  let mut stats = Stats::default();
  reduce_with_stats(&mut net, &Bubble, &mut stats);
  println!("{stats}");
}
