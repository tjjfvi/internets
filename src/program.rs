use std::{
  collections::{hash_map::Entry, HashMap},
  ops::Range,
};

use itertools::Itertools;

use crate::parser;

#[derive(Debug)]
pub struct Program {
  pub types: Vec<Type>,
  pub nodes: Vec<Node>,
  pub init: Init,
}

#[derive(Debug)]
pub struct Type {
  pub id: u32,
  pub name: String,
  pub nodes: Range<u32>,
}

#[derive(Debug)]
pub struct Node {
  pub id: u32,
  pub arity: u32,
  pub name: String,
  pub args: Vec<(bool, u32)>,
  pub rules: HashMap<u32, Rule>,
}

#[derive(Debug)]
pub struct Rule {
  pub vars: u32,
  pub left: Vec<Net>,
  pub right: Vec<Net>,
}

#[derive(Debug)]
pub enum Net {
  Node(u32, u32, Vec<Net>),
  Var(u32),
}

#[derive(Debug)]
pub struct Init {
  pub vars: u32,
  pub nets: Vec<(u32, Net)>,
  pub pins: Vec<u32>,
}

pub fn build_program(ast: &parser::Program) -> Program {
  let mut types = Vec::new();
  let mut nodes = Vec::new();
  let mut type_ids = HashMap::new();
  let mut node_ids = HashMap::new();
  for type_ast in &ast.0 {
    let id = types.len() as u32;
    let nodes_start = nodes.len() as u32;
    let nodes_end = nodes_start + type_ast.1.len() as u32;
    types.push(Type {
      id,
      name: type_ast.0.to_owned(),
      nodes: nodes_start..nodes_end,
    });
    type_ids.insert(type_ast.0, id);
    for node_ast in &type_ast.1 {
      let id = nodes.len() as u32;
      nodes.push(Node {
        id,
        arity: node_ast.1.len() as u32,
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
        for (var, twice) in vars.values() {
          if !twice {
            panic!("var {} only used once", var)
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
    let parser::RuleNet::Node(..) = right else {
      panic!("expected right side of init to be node")
    };
    let mut cb = |ast| build_rule_node(ast, &nodes, &node_ids, &mut init_vars, &mut init_var_count);
    let left = cb(left);
    let right = cb(right);
    match &left {
      &Net::Node(..) => {
        let var = init_var_count;
        init_var_count += 1;
        init_nets.push((var, left));
        init_nets.push((var, right))
      }
      &Net::Var(var) => init_nets.push((var, right)),
    }
  }
  Program {
    types,
    nodes,
    init: Init {
      vars: init_var_count,
      nets: init_nets,
      pins: init_vars.values().filter(|x| !x.1).map(|x| x.0).collect(),
    },
  }
}

fn build_rule_node<'a>(
  ast: &parser::RuleNet<'a>,
  nodes: &Vec<Node>,
  node_ids: &HashMap<&'a str, u32>,
  vars: &mut HashMap<&'a str, (u32, bool)>,
  var_count: &mut u32,
) -> Net {
  match ast {
    parser::RuleNet::Var(var) => match vars.entry(var) {
      Entry::Occupied(mut e) => {
        let e = e.get_mut();
        if e.1 {
          panic!("var {} used more than twice", var)
        }
        e.1 = true;
        Net::Var(e.0)
      }
      Entry::Vacant(e) => {
        let id = *var_count;
        *var_count += 1;
        e.insert((id, false));
        Net::Var(id)
      }
    },
    parser::RuleNet::Node(name, children) => {
      let id = node_ids[name];
      let arity = nodes[id as usize].arity;
      Net::Node(
        id,
        arity,
        children
          .iter()
          .map(|ast| build_rule_node(ast, nodes, node_ids, vars, var_count))
          .collect(),
      )
    }
  }
}
