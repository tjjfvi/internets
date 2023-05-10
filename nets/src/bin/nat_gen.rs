use internets_nets::interactions;

mod libs;

interactions! {
  use libs::nat;

  fn square(i: -U64, o: +U64) {
    Clone(i, i0, i1)
    Mul(i0, i1, o)
  }

  fn _main(n65536: +U64) {
    Zero(n0)
    Succ(n1, n0)
    Succ(n2, n1)
    square(n2, n4)
    square(n4, n16)
    square(n16, n256)
    square(n256, n65536)
  }
}

fn main() {
  use internets_nets::*;
  let mut stats = Stats::default();
  let mut buffer = ArrayBuffer::new(1 << 18);
  for _ in 0..1000 {
    let mut net = BasicNet::new(LinkAlloc::new(buffer.as_mut()));
    let free = net.alloc_write(&[Word::NULL]);
    let mut free_0 = LinkHalf::Null;
    _main(&mut free_0).construct(&mut net, &Interactions);
    net.link(free_0, LinkHalf::Port(free, PortMode::Auxiliary));
    reduce_with_stats(&mut net, &Interactions, &mut stats);
  }
  eprintln!("{stats}");
}
