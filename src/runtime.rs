use std::{fmt::Write, iter};

use crate::program::*;

#[derive(Clone, Copy)]
pub struct Port(u32);

impl Port {
  pub fn is_principal(&self) -> bool {
    self.0 & 1 != 0
  }
  pub fn adr(&self) -> usize {
    (self.0 >> 1) as usize
  }
  pub fn new_null(adr: usize) -> Port {
    Port((adr as u32) << 1)
  }
  pub fn new_aux(adr: usize) -> Port {
    Port((adr as u32) << 1)
  }
  pub fn new_principal(adr: usize) -> Port {
    Port(((adr as u32) << 1) | 1)
  }
  pub fn arg(&self, i: usize) -> Port {
    Port::new_aux(self.adr() + 1 + i)
  }
}

pub struct Runtime<'a> {
  program: &'a Program,
  mem: Vec<Port>,
  vars: Vec<Port>,
  free: Vec<usize>,
  active: Vec<(Port, Port)>,
  steps: u64,
}

impl<'a> Runtime<'a> {
  pub fn get(&self, port: Port) -> Port {
    self.mem[port.adr()]
  }
  pub fn kind(&self, port: Port) -> usize {
    self.mem[port.adr()].adr()
  }
  pub fn alloc(&mut self, kind: usize, arity: usize) -> Port {
    if arity == 0 {
      return Port::new_principal(kind);
    }
    let free_adr = &mut self.free[arity - 1];
    let adr = if *free_adr > self.mem.len() {
      let start = self.mem.len();
      self
        .mem
        .extend(std::iter::repeat(Port::new_null(0)).take(arity + 1));
      start
    } else {
      std::mem::replace(free_adr, self.mem[*free_adr].adr())
    };
    self.mem[adr] = Port::new_principal(kind);
    Port::new_principal(adr)
  }
  pub fn free(&mut self, port: Port, arity: usize) {
    if arity == 0 {
      return;
    }
    let adr = port.adr();
    self.mem[adr] = Port::new_null(std::mem::replace(&mut self.free[arity - 1], adr));
    for i in 0..arity {
      self.mem[adr + 1 + i] = Port::new_null(usize::MAX);
    }
  }
  pub fn link(&mut self, p: Port, q: Port) {
    if !p.is_principal() {
      self.mem[p.adr()] = q;
    }
    if !q.is_principal() {
      self.mem[q.adr()] = p;
    }
    if p.is_principal() && q.is_principal() {
      self.active.push((p, q))
    }
  }

  pub fn new(program: &Program) -> Runtime {
    let mut runtime = Runtime {
      program,
      mem: (0..program.kinds.len())
        .map(|x| Port::new_principal(x))
        .chain(iter::repeat(Port::new_null(0)).take(program.init.pins.len()))
        .collect(),
      vars: vec![Port::new_null(0); program.max_vars()],
      free: vec![usize::MAX; program.max_arity()],
      active: vec![],
      steps: 0,
    };
    for (node, net) in &program.init.nets {
      let port = runtime.create_node(node);
      runtime.graft_net(port, net);
    }
    for (i, &var) in program.init.pins.iter().enumerate() {
      runtime.link(runtime.free_port(i), runtime.vars[var]);
    }
    runtime
  }
  pub fn free_port(&self, i: usize) -> Port {
    Port::new_aux(self.program.kinds.len() + i)
  }
  pub fn graft_net(&mut self, port: Port, net: &Net) {
    match net {
      &Net::Var(var) => {
        if var.def {
          self.link(port, self.vars[var.id]);
        } else {
          self.vars[var.id] = port;
        }
      }
      &Net::Node(ref node) => {
        let node = self.create_node(node);
        self.link(port, node);
      }
    }
  }
  pub fn create_node(&mut self, node: &Node) -> Port {
    let port = self.alloc(node.kind, node.arity);
    for (i, child) in node.ports.iter().enumerate() {
      self.graft_net(port.arg(i), child)
    }
    port
  }
  pub fn normal(&self) -> bool {
    self.active.is_empty()
  }
  pub fn reduce(&mut self) {
    self.steps += 1;
    let (port1, port2) = self.active.pop().unwrap();
    let kind1 = self.kind(port1);
    let kind2 = self.kind(port2);
    let ((port1, nets1), (port2, nets2)) = self.program.kinds[kind1]
      .rules
      .get(&kind2)
      .map(|x| ((port2, &x.left), (port1, &x.right)))
      .or_else(|| {
        self.program.kinds[kind2]
          .rules
          .get(&kind1)
          .map(|x| ((port1, &x.left), (port2, &x.right)))
      })
      .unwrap();
    for (i, net) in nets1.iter().enumerate() {
      let dst = self.get(port1.arg(i));
      self.graft_net(dst, net);
    }
    for (i, net) in nets2.iter().enumerate() {
      let dst = self.get(port2.arg(i));
      self.graft_net(dst, net);
    }
    self.free(port1, nets1.len());
    self.free(port2, nets2.len());
  }

  pub fn dbg_port<'b>(&'b self, port: Port) -> impl std::fmt::Debug + 'b {
    DebugFn(move |f| {
      if port.is_principal() {
        let kind = self.kind(port);
        if kind == port.adr() {
          write!(f, "{}", &self.program.kinds[kind].name,)
        } else {
          write!(f, "{} {}", &self.program.kinds[kind].name, port.adr())
        }
      } else {
        write!(f, "{}", port.adr())
      }
    })
  }
  pub fn dbg_mem<'b>(&'b self, vec: &'b Vec<Port>) -> impl std::fmt::Debug + 'b {
    DebugFn(|f| {
      let mut f = f.debug_map();
      let mut arity_left = 0;
      for (i, &port) in vec.iter().enumerate() {
        if arity_left > 0 {
          arity_left -= 1;
          f.entry(&i, &DebugFn(|f| write!(f, "  {:?}", self.dbg_port(port))));
        } else if port.is_principal() && self.kind(port) == port.adr() && i != port.adr() {
          let kind = &self.program.kinds[self.kind(port)];
          arity_left = kind.arity;
          f.entry(&i, &DebugFn(|f| write!(f, "{}", kind.name)));
        }
      }
      f.finish()
    })
  }
  pub fn dbg_tree<'b>(&'b self, port: Port) -> impl std::fmt::Debug + 'b {
    DebugFn(move |f| {
      write!(f, "{:?}", self.dbg_port(port))?;
      if !port.is_principal() {
        return Ok(());
      }
      let node = &self.program.kinds[self.kind(port)];
      if node.arity == 0 {
        return Ok(());
      }
      f.write_char(' ')?;
      let mut f = f.debug_list();
      for i in 0..node.arity {
        f.entry(&self.dbg_tree(self.get(port.arg(i))));
      }
      f.finish()
    })
  }
}

struct DebugAsStr(String);
struct DebugFn<F: Fn(&mut std::fmt::Formatter<'_>) -> std::fmt::Result>(F);

impl std::fmt::Debug for DebugAsStr {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.0)
  }
}

impl<F: Fn(&mut std::fmt::Formatter<'_>) -> std::fmt::Result> std::fmt::Debug for DebugFn<F> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.0(f)
  }
}

impl<'a> std::fmt::Debug for Runtime<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut f = f.debug_struct("Runtime");
    f.field("mem", &self.dbg_mem(&self.mem));
    // f.field("vars", &self.dbg_port_vec(self.vars));
    // f.field("work", &self.dbg_port_vec(&self.work));
    f.field("gens", &self.steps);
    f.finish()
  }
}
