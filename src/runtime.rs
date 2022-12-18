use std::fmt::Write;

use crate::program::*;

#[derive(Clone, Copy)]
pub struct Port(u64);

impl Port {
  pub fn is_principal(&self) -> bool {
    self.0 & 1 != 0
  }
  pub fn is_null(&self) -> bool {
    self.0 & 3 == 0
  }
  pub fn is_auxillary(&self) -> bool {
    self.0 & 3 == 2
  }
  pub fn kind(&self) -> usize {
    (self.0 as u32 >> 1) as usize
  }
  pub fn adr(&self) -> usize {
    (self.0 >> 32) as usize
  }
  pub fn new_null(adr: usize) -> Port {
    Port((adr as u64) << 32)
  }
  pub fn new_aux(adr: usize) -> Port {
    Port(((adr as u64) << 32) | 0b10)
  }
  pub fn new_principal(kind: usize, adr: usize) -> Port {
    Port(((adr as u64) << 32) | ((kind as u32) << 1) as u64 | 1)
  }
  pub fn arg(&self, i: usize) -> Port {
    Port::new_aux(self.adr() + i)
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
  pub fn get(&self, port: Port, i: usize) -> Port {
    self.mem[port.adr() + i]
  }
  pub fn alloc(&mut self, kind: usize, arity: usize) -> Port {
    if arity == 0 {
      return Port::new_principal(kind, 0);
    }
    let free_adr = &mut self.free[arity - 1];
    let adr = if *free_adr > self.mem.len() {
      let start = self.mem.len();
      self
        .mem
        .extend(std::iter::repeat(Port::new_null(0)).take(arity));
      start
    } else {
      std::mem::replace(free_adr, self.mem[*free_adr].adr())
    };
    Port::new_principal(kind, adr)
  }
  pub fn free(&mut self, port: Port, arity: usize) {
    if arity == 0 {
      return;
    }
    let adr = port.adr();
    self.mem[adr] = Port::new_null(std::mem::replace(&mut self.free[arity - 1], adr));
    for i in 0..arity - 1 {
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
      mem: vec![Port::new_null(0); program.init.pins.len()],
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
      runtime.link(Port::new_aux(i), runtime.vars[var]);
    }
    runtime
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
    let ((port1, nets1), (port2, nets2)) = self.program.kinds[port1.kind()]
      .rules
      .get(&port2.kind())
      .map(|x| ((port2, &x.left), (port1, &x.right)))
      .or_else(|| {
        self.program.kinds[port2.kind()]
          .rules
          .get(&port1.kind())
          .map(|x| ((port1, &x.left), (port2, &x.right)))
      })
      .unwrap();
    for (i, net) in nets1.iter().enumerate() {
      let dst = self.get(port1, i);
      self.graft_net(dst, net);
    }
    for (i, net) in nets2.iter().enumerate() {
      let dst = self.get(port2, i);
      self.graft_net(dst, net);
    }
    self.free(port1, nets1.len());
    self.free(port2, nets2.len());
  }

  pub fn dbg_port(&self, port: Port) -> impl std::fmt::Debug {
    DebugAsStr(if port.is_null() {
      format!("null {}", port.adr())
    } else if port.is_auxillary() {
      format!("aux {}", port.adr())
    } else {
      let node = &self.program.kinds[port.kind()];
      if node.arity == 0 {
        format!("{}", &node.name)
      } else {
        format!("{} {:?}", &node.name, port.adr()..port.adr() + node.arity)
      }
    })
  }
  pub fn dbg_port_vec<'b>(&'b self, vec: &'b Vec<Port>) -> impl std::fmt::Debug + 'b {
    DebugFn(|f| {
      let mut f = f.debug_map();
      for (i, &port) in vec.iter().enumerate() {
        f.entry(&i, &self.dbg_port(port));
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
      let node = &self.program.kinds[port.kind()];
      if node.arity == 0 {
        return Ok(());
      }
      f.write_char(' ')?;
      let mut f = f.debug_list();
      for i in 0..node.arity {
        f.entry(&self.dbg_tree(self.get(port, i)));
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
    f.field("mem", &self.dbg_port_vec(&self.mem));
    f.field("vars", &self.dbg_port_vec(&self.vars));
    // f.field("work", &self.dbg_port_vec(&self.work));
    f.field("gens", &self.steps);
    f.finish()
  }
}
