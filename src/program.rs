use std::{
  collections::{hash_map::Entry, HashMap},
  ops::Range,
};

use itertools::Itertools;

use crate::parser;

#[derive(Debug)]
pub struct Program {
  pub types: Vec<Type>,
  pub kinds: Vec<Kind>,
  pub init: Init,
}

#[derive(Debug)]
pub struct Type {
  pub id: usize,
  pub name: String,
  pub nodes: Range<usize>,
}

#[derive(Debug)]
pub struct Kind {
  pub id: usize,
  pub arity: usize,
  pub name: String,
  pub args: Vec<(bool, usize)>,
  pub rules: HashMap<usize, Rule>,
}

#[derive(Debug)]
pub struct Rule {
  pub vars: usize,
  pub left: Vec<Net>,
  pub right: Vec<Net>,
}

#[derive(Debug)]
pub enum Net {
  Node(Node),
  Var(Var),
}

#[derive(Debug, Clone, Copy)]
pub struct Var {
  pub id: usize,
  pub def: bool,
}

#[derive(Debug)]
pub struct Node {
  pub kind: usize,
  pub arity: usize,
  pub ports: Vec<Net>,
}

#[derive(Debug)]
pub struct Init {
  pub vars: usize,
  pub nets: Vec<(Node, Net)>,
  pub free: Vec<usize>,
}

pub fn build_program(ast: &parser::Program) -> Program {
  let mut types = Vec::new();
  let mut nodes = Vec::new();
  let mut type_ids = HashMap::new();
  let mut node_ids = HashMap::new();
  for type_ast in &ast.0 {
    let id = types.len();
    let nodes_start = nodes.len();
    let nodes_end = nodes_start + type_ast.1.len();
    types.push(Type {
      id,
      name: type_ast.0.to_owned(),
      nodes: nodes_start..nodes_end,
    });
    type_ids.insert(type_ast.0, id);
    for node_ast in &type_ast.1 {
      let id = nodes.len();
      nodes.push(Kind {
        id,
        arity: node_ast.1.len(),
        name: node_ast.0.to_owned(),
        args: Vec::with_capacity(node_ast.1.len()),
        rules: HashMap::new(),
      });
      node_ids.insert(node_ast.0, id);
    }
  }
  for type_ast in &ast.0 {
    for node_ast in &type_ast.1 {
      let id = node_ids[node_ast.0];
      let node = &mut nodes[id as usize];
      for &(sign, other) in &node_ast.1 {
        node.args.push((sign, type_ids[other]));
      }
      for rule_ast in &node_ast.2 {
        let parser::RuleNet::Node(left_name, left_children) = &rule_ast.0 else {
          panic!("cannot use var directly in rule")
        };
        let parser::RuleNet::Node(right_name, right_children) = &rule_ast.1 else {
          panic!("cannot use var directly in rule")
        };
        if right_name != &node_ast.0 {
          panic!("expected right side of rule to be the parent")
        }
        dbg!(&left_name);
        let other = node_ids[left_name];
        let mut vars = HashMap::new();
        let mut var_count = 0;
        let mut cb = |ast| build_rule_node(ast, &nodes, &node_ids, &mut vars, &mut var_count);
        let left = left_children.iter().map(&mut cb).collect_vec();
        let right = right_children.iter().map(&mut cb).collect_vec();
        for (name, var) in vars {
          if !var.def {
            panic!("var {} only used once", name)
          }
        }
        let node = &mut nodes[id as usize];
        node.rules.insert(
          other,
          Rule {
            vars: var_count,
            left,
            right,
          },
        );
      }
    }
  }
  let mut init_nets = Vec::new();
  let mut init_vars = HashMap::new();
  let mut init_var_count = 0;
  for (left, right) in &ast.1 .0 {
    let mut cb = |ast| build_rule_node(ast, &nodes, &node_ids, &mut init_vars, &mut init_var_count);
    let Net::Node(left) = cb(left) else {
      panic!("expected left side to be a node")
    };
    let right = cb(right);
    init_nets.push((left, right));
  }
  Program {
    types,
    kinds: nodes,
    init: Init {
      vars: init_var_count,
      nets: init_nets,
      free: init_vars
        .values()
        .filter(|x| !x.def)
        .map(|x| x.id)
        .collect(),
    },
  }
}

fn build_rule_node<'a>(
  ast: &parser::RuleNet<'a>,
  nodes: &Vec<Kind>,
  node_ids: &HashMap<&'a str, usize>,
  vars: &mut HashMap<&'a str, Var>,
  var_count: &mut usize,
) -> Net {
  match ast {
    parser::RuleNet::Var(name) => match vars.entry(name) {
      Entry::Occupied(mut e) => {
        let var = e.get_mut();
        if var.def {
          panic!("var {} used more than twice", name)
        }
        var.def = true;
        Net::Var(*var)
      }
      Entry::Vacant(e) => {
        let var = Var {
          id: *var_count,
          def: false,
        };
        *var_count += 1;
        e.insert(var);
        Net::Var(var)
      }
    },
    parser::RuleNet::Node(name, ports) => {
      let kind = node_ids[name];
      let arity = nodes[kind as usize].arity;
      Net::Node(Node {
        kind,
        arity,
        ports: ports
          .iter()
          .map(|ast| build_rule_node(ast, nodes, node_ids, vars, var_count))
          .collect(),
      })
    }
  }
}

impl Program {
  pub fn max_arity(&self) -> usize {
    self.kinds.iter().map(|x| x.arity).max().unwrap_or(0)
  }
  pub fn max_vars(&self) -> usize {
    self
      .kinds
      .iter()
      .flat_map(|x| x.rules.values())
      .map(|x| x.vars)
      .max()
      .unwrap_or(0)
      .max(self.init.vars)
  }
}
