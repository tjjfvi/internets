use crate::program::*;

pub type Ptr = (Tag, u32);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Tag(pub i32);

impl Tag {
  pub const NUL: Tag = Tag(-1);
  pub const AUX: Tag = Tag(-2);
  pub const PIN: Tag = Tag(-3);
  pub fn ok(&self) -> bool {
    self.0 >= 0
  }
}

pub struct Runtime<'a> {
  program: &'a Program,
  mem: Vec<Ptr>,
  vars: Vec<Ptr>,
  free: Vec<u32>,
  work: Vec<Ptr>,
  gens: u64,
}

impl<'a> Runtime<'a> {
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
    let mut mem = Runtime {
      program,
      mem: vec![(Tag::NUL, 0); program.init.pins.len().max(1)],
      vars: vec![(Tag::NUL, u32::MAX); max_vars as usize],
      free: vec![u32::MAX; max_arity as usize],
      work: vec![],
      gens: 0,
    };
    for &(var, ref net) in &program.init.nets {
      dbg!(&mem, net);
      mem.instantiate_net((Tag::AUX, 0), net);
      dbg!(&mem);
      mem.instantiate_net(mem.get(0), &Net::Var(var));
    }
    for (i, &var) in program.init.pins.iter().enumerate() {
      dbg!(&mem);
      mem.instantiate_net((Tag::PIN, i as u32), &Net::Var(var));
    }
    mem
  }
  pub fn get(&self, adr: u32) -> Ptr {
    self.mem[adr as usize]
  }
  pub fn get_mut(&mut self, adr: u32) -> &mut Ptr {
    &mut self.mem[adr as usize]
  }
  pub fn alloc(&mut self, arity: u8) -> u32 {
    let arity = arity as usize;
    let free_adr = &mut self.free[arity - 1];
    if free_adr == &u32::MAX {
      let start = self.mem.len();
      self
        .mem
        .extend(std::iter::repeat((Tag::NUL, 0)).take(arity));
      return start as u32;
    }
    let start = *free_adr;
    *free_adr = self.mem[start as usize].1;
    start
  }
  pub fn free(&mut self, adr: u32, arity: u8) {
    let arity = arity as usize;
    let free_adr = &mut self.free[arity - 1];
    self.mem[adr as usize] = (Tag::NUL, *free_adr);
    for i in 1..arity {
      self.mem[adr as usize + i] = (Tag::NUL, u32::MAX);
    }
    *free_adr = adr;
  }
  pub fn instantiate_net(&mut self, ptr: Ptr, net: &Net) {
    match net {
      &Net::Var(var) => {
        let var = &mut self.vars[var as usize];
        if var.0 == Tag::NUL {
          *var = ptr;
        } else {
          let var = std::mem::replace(var, (Tag::NUL, u32::MAX));
          *self.get_mut(var.1) = ptr;
          *self.get_mut(ptr.1) = var;
        }
      }
      &Net::Node(kind, arity, ref children) => {
        let mut adr = self.alloc(arity);
        *self.get_mut(ptr.1) = (Tag(kind), adr);
        *self.get_mut(adr) = ptr;
        for child in children {
          adr += 1;
          self.instantiate_net((Tag::AUX, adr), child)
        }
      }
    }
    if ptr.0.ok() && self.get(ptr.1).0.ok() {
      self.work.push(ptr)
    }
  }
  pub fn dbg_ptr(&self, ptr: Ptr) -> impl std::fmt::Debug {
    DebugAsStr(if ptr.0 == Tag::NUL {
      format!("_")
    } else if ptr.0 == Tag::AUX {
      format!("AUX {}", ptr.1)
    } else if ptr.0 == Tag::PIN {
      format!("PIN {}", ptr.1)
    } else {
      let node = &self.program.nodes[ptr.0 .0 as usize];
      if node.arity == 1 {
        format!("{} {}", &node.name, ptr.1)
      } else {
        format!(
          "{} {:?}",
          &node.name,
          ptr.1..=ptr.1 + (node.arity as u32) - 1
        )
      }
    })
  }
  pub fn dbg_vec_ptr<'b>(&'b self, vec: &'b Vec<Ptr>) -> impl std::fmt::Debug + 'b {
    DebugFn(|f| {
      let mut f = f.debug_map();
      for (i, &ptr) in vec.iter().enumerate() {
        f.entry(&i, &self.dbg_ptr(ptr));
      }
      f.finish()
    })
  }
  pub fn has_work(&self) -> bool {
    !self.work.is_empty()
  }
  pub fn step(&mut self) {
    self.gens += 1;
    let ptr1 = self.work.pop().unwrap();
    let ptr2 = self.get(ptr1.1);
    let rule = self.program.nodes[ptr1.0 .0 as usize]
      .rules
      .get(&ptr2.0 .0)
      .map(|x| (&x.right, &x.left))
      .or_else(|| {
        self.program.nodes[ptr2.0 .0 as usize]
          .rules
          .get(&ptr1.0 .0)
          .map(|x| (&x.left, &x.right))
      })
      .unwrap();
    for (i, net) in rule.0.iter().enumerate() {
      let dst = self.get(ptr1.1 + 1 + i as u32);
      self.instantiate_net(dst, net);
    }
    for (i, net) in rule.1.iter().enumerate() {
      let dst = self.get(ptr2.1 + 1 + i as u32);
      self.instantiate_net(dst, net);
    }
    self.free(ptr1.1, (rule.0.len() + 1) as u8);
    self.free(ptr2.1, (rule.1.len() + 1) as u8);
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
    f.field("mem", &self.dbg_vec_ptr(&self.mem));
    f.field("vars", &self.dbg_vec_ptr(&self.vars));
    f.field("work", &self.dbg_vec_ptr(&self.work));
    f.field("gens", &self.gens);
    f.finish()
  }
}
