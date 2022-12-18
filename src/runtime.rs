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
  pub fn kind(&self) -> u32 {
    self.0 as u32 >> 1
  }
  pub fn adr(&self) -> u32 {
    (self.0 >> 32) as u32
  }
  pub fn new_null(adr: u32) -> Port {
    Port((adr as u64) << 32)
  }
  pub fn new_aux(adr: u32) -> Port {
    Port(((adr as u64) << 32) | 0b10)
  }
  pub fn new_principal(kind: u32, adr: u32) -> Port {
    Port(((adr as u64) << 32) | (kind << 1) as u64 | 1)
  }
  pub fn arg(&self, i: u32) -> Port {
    Port::new_aux(self.adr() + i)
  }
}

pub struct Runtime<'a> {
  program: &'a Program,
  mem: Vec<Port>,
  vars: Vec<Port>,
  free: Vec<u32>,
  active: Vec<(Port, Port)>,
  steps: u64,
}

impl<'a> Runtime<'a> {
  pub fn get(&self, port: Port, i: u32) -> Port {
    self.mem[(port.adr() + i) as usize]
  }
  pub fn alloc(&mut self, kind: u32, arity: u32) -> Port {
    if arity == 0 {
      return Port::new_principal(kind, u32::MAX);
    }
    let arity = arity as usize;
    let free_adr = &mut self.free[arity - 1];
    let adr = if free_adr == &u32::MAX {
      let start = self.mem.len();
      self
        .mem
        .extend(std::iter::repeat(Port::new_null(0)).take(arity));
      start as u32
    } else {
      let start = *free_adr;
      *free_adr = self.mem[start as usize].adr();
      start
    };
    Port::new_principal(kind, adr)
  }
  pub fn link(&mut self, p: Port, q: Port) {
    if !p.is_principal() {
      self.mem[p.adr() as usize] = q;
    }
    if !q.is_principal() {
      self.mem[q.adr() as usize] = p;
    }
  }
  pub fn free(&mut self, port: Port, arity: u32) {
    if arity == 0 {
      return;
    }
    let arity = arity as usize;
    let adr = port.adr();
    let free_adr = &mut self.free[arity - 1];
    self.mem[adr as usize] = Port::new_null(*free_adr);
    for i in 0..arity - 1 {
      self.mem[adr as usize + 1 + i] = Port::new_null(u32::MAX);
    }
    *free_adr = adr;
  }

  pub fn new(program: &Program) -> Runtime {
    let max_arity = program.nodes.iter().map(|x| x.arity).max().unwrap_or(0);
    let max_vars = program
      .nodes
      .iter()
      .flat_map(|x| x.rules.values())
      .map(|x| x.vars)
      .max()
      .unwrap_or(0)
      .max(program.init.vars);
    let mut runtime = Runtime {
      program,
      mem: vec![Port::new_null(0); program.init.nets.len().max(program.init.pins.len())],
      vars: vec![Port::new_null(0); max_vars as usize],
      free: vec![u32::MAX; max_arity as usize],
      active: vec![],
      steps: 0,
    };
    for (i, &(var, ref net)) in program.init.nets.iter().enumerate() {
      let port = Port::new_aux(i as u32);
      runtime.graft(port, net);
      runtime.graft(runtime.get(port, 0), &Net::Var(var));
    }
    for (i, &var) in program.init.pins.iter().enumerate() {
      runtime.graft(Port::new_aux(i as u32), &Net::Var(var));
    }
    runtime
  }
  pub fn graft(&mut self, port: Port, net: &Net) {
    let other = match net {
      &Net::Var(var) => {
        let var = &mut self.vars[var as usize];
        if var.is_null() {
          *var = port;
          return;
        }
        std::mem::replace(var, Port::new_null(0))
      }
      &Net::Node(kind, arity, ref children) => {
        let node = self.alloc(kind, arity);
        for (i, child) in children.iter().enumerate() {
          self.graft(node.arg(i as u32), child)
        }
        node
      }
    };
    self.link(port, other);
    if port.is_principal() && other.is_principal() {
      self.active.push((port, other))
    }
  }
  pub fn normal(&self) -> bool {
    self.active.is_empty()
  }
  pub fn reduce(&mut self) {
    self.steps += 1;
    let (port1, port2) = self.active.pop().unwrap();
    let rule = self.program.nodes[port1.kind() as usize]
      .rules
      .get(&port2.kind())
      .map(|x| (&x.right, &x.left))
      .or_else(|| {
        self.program.nodes[port2.kind() as usize]
          .rules
          .get(&port1.kind())
          .map(|x| (&x.left, &x.right))
      })
      .unwrap();
    for (i, net) in rule.0.iter().enumerate() {
      let dst = self.get(port1, i as u32);
      self.graft(dst, net);
    }
    for (i, net) in rule.1.iter().enumerate() {
      let dst = self.get(port2, i as u32);
      self.graft(dst, net);
    }
    self.free(port1, rule.0.len() as u32);
    self.free(port2, rule.1.len() as u32);
  }

  pub fn dbg_port(&self, port: Port) -> impl std::fmt::Debug {
    DebugAsStr(if port.is_null() {
      format!("null {}", port.adr())
    } else if port.is_auxillary() {
      format!("aux {}", port.adr())
    } else {
      let node = &self.program.nodes[port.kind() as usize];
      if node.arity == 0 {
        format!("{}", &node.name)
      } else {
        format!(
          "{} {:?}",
          &node.name,
          port.adr()..port.adr() + (node.arity as u32)
        )
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
      let node = &self.program.nodes[port.kind() as usize];
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
