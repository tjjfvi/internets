use itertools::Itertools;
use std::iter::Peekable;

#[derive(Debug, Clone, Copy)]
enum Token<'a> {
  Node,
  Type,
  Rule,
  Init,
  Name(&'a str),
  Var(&'a str),
  BracketL,
  BracketR,
  CurlyL,
  CurlyR,
  Sign(bool),
}

fn lex(source: &str) -> impl Iterator<Item = Token> {
  source
    .char_indices()
    .chain([(0, '\0')])
    .tuple_windows::<(_, _)>()
    .flat_map({
      let mut word_data = Option::<(usize, bool)>::None;
      move |((i, char), (_, next))| match char {
        '[' => Some(Token::BracketL),
        ']' => Some(Token::BracketR),
        '{' => Some(Token::CurlyL),
        '}' => Some(Token::CurlyR),
        '+' => Some(Token::Sign(true)),
        '-' => Some(Token::Sign(false)),
        x if x.is_alphanumeric() => {
          word_data = word_data.or(Some((i, x.is_uppercase())));
          if next.is_alphanumeric() {
            None
          } else {
            let (start, upper) = word_data.take().unwrap();
            let str = &source[start..=i];
            Some(if upper {
              Token::Name(str)
            } else if str == "node" {
              Token::Node
            } else if str == "type" {
              Token::Type
            } else if str == "rule" {
              Token::Rule
            } else if str == "init" {
              Token::Init
            } else {
              Token::Var(str)
            })
          }
        }
        x if x.is_whitespace() => None,
        _ => panic!("invalid char"),
      }
    })
}

#[derive(Debug)]
pub enum RuleNet<'a> {
  Node(&'a str, Vec<RuleNet<'a>>),
  Var(&'a str),
}

fn parse_rule_net<'a>(tokens: &mut Peekable<impl Iterator<Item = Token<'a>>>) -> RuleNet<'a> {
  match tokens.next() {
    Some(Token::Name(atom)) => RuleNet::Node(atom, Vec::new()),
    Some(Token::Var(var)) => RuleNet::Var(var),
    Some(Token::BracketL) => {
      let atom = match tokens.next() {
        Some(Token::Name(atom)) => atom,
        x => panic!("expected name, got {:?}", x),
      };
      dbg!(&atom);
      let mut children = Vec::new();
      while !matches!(tokens.peek(), Some(Token::BracketR)) {
        children.push(parse_rule_net(tokens));
      }
      tokens.next();
      RuleNet::Node(atom, children)
    }
    x => panic!("expected name, var, or `[`, got {:?}", x),
  }
}

#[derive(Debug)]
pub struct Rule<'a>(pub RuleNet<'a>, pub RuleNet<'a>);

fn parse_rule<'a>(tokens: &mut Peekable<impl Iterator<Item = Token<'a>>>) -> Rule<'a> {
  match tokens.next() {
    Some(Token::Rule) => {}
    x => panic!("expected `rule`, got {:?}", x),
  }
  Rule(parse_rule_net(tokens), parse_rule_net(tokens))
}

fn parse_group<'a, I: Iterator<Item = Token<'a>>, T>(
  tokens: &mut Peekable<I>,
  parse_item: fn(&mut Peekable<I>) -> T,
) -> (&'a str, Vec<T>) {
  match tokens.next() {
    Some(Token::Name(atom)) => (atom, Vec::new()),
    Some(Token::BracketL) => {
      let atom = match tokens.next() {
        Some(Token::Name(atom)) => atom,
        x => panic!("expected name, got {:?}", x),
      };
      let mut parts = Vec::new();
      while !matches!(tokens.peek(), Some(Token::BracketR)) {
        parts.push(parse_item(tokens))
      }
      tokens.next();
      (atom, parts)
    }
    x => panic!("expected name or `[`, got {:?}", x),
  }
}

fn parse_block<'a, I: Iterator<Item = Token<'a>>, T>(
  tokens: &mut Peekable<I>,
  parse_item: fn(&mut Peekable<I>) -> T,
) -> Vec<T> {
  match tokens.peek() {
    Some(Token::CurlyL) => tokens.next(),
    _ => return Vec::new(),
  };
  let mut items = Vec::new();
  while !matches!(tokens.peek(), Some(Token::CurlyR)) {
    items.push(parse_item(tokens));
  }
  tokens.next();
  items
}

#[derive(Debug)]
pub struct Node<'a>(pub &'a str, pub Vec<(bool, &'a str)>, pub Vec<Rule<'a>>);

fn parse_node<'a>(tokens: &mut Peekable<impl Iterator<Item = Token<'a>>>) -> Node<'a> {
  match tokens.next() {
    Some(Token::Node) => {}
    x => panic!("expected `node`, got {:?}", x),
  };
  let (atom, items) = parse_group(tokens, |tokens| {
    let sign = match tokens.next() {
      Some(Token::Sign(sign)) => sign,
      x => panic!("expected sign, got {:?}", x),
    };
    let name = match tokens.next() {
      Some(Token::Name(name)) => name,
      x => panic!("expected type, got {:?}", x),
    };
    (sign, name)
  });
  dbg!(&atom);
  let rules = parse_block(tokens, parse_rule);
  Node(atom, items, rules)
}

#[derive(Debug)]
pub struct Type<'a>(pub &'a str, pub Vec<Node<'a>>);

fn parse_type<'a>(tokens: &mut Peekable<impl Iterator<Item = Token<'a>>>) -> Type<'a> {
  match tokens.peek() {
    Some(Token::Node) => {
      let node = parse_node(tokens);
      return Type(node.0, vec![node]);
    }
    Some(Token::Type) => tokens.next(),
    x => panic!("expected `node` or `type`, got {:?}", x),
  };
  let name = match tokens.next() {
    Some(Token::Name(name)) => name,
    x => panic!("expected name, got {:?}", x),
  };
  let nodes = parse_block(tokens, parse_node);
  Type(name, nodes)
}

#[derive(Debug)]
pub struct Init<'a>(pub Vec<(RuleNet<'a>, RuleNet<'a>)>);

fn parse_init<'a>(tokens: &mut Peekable<impl Iterator<Item = Token<'a>>>) -> Init<'a> {
  match tokens.next() {
    Some(Token::Init) => {}
    x => panic!("expected `init`, got {:?}", x),
  };
  Init(parse_block(tokens, |tokens| {
    (parse_rule_net(tokens), parse_rule_net(tokens))
  }))
}

#[derive(Debug)]
pub struct Program<'a>(pub Vec<Type<'a>>, pub Init<'a>);

fn parse_program<'a>(tokens: &mut Peekable<impl Iterator<Item = Token<'a>>>) -> Program<'a> {
  let mut types = Vec::new();
  while !matches!(tokens.peek(), Some(Token::Init)) {
    types.push(parse_type(tokens));
  }
  let init = parse_init(tokens);
  if tokens.peek().is_some() {
    panic!("expected eof, got {:?}", tokens.next());
  }
  Program(types, init)
}

pub fn parse(source: &str) -> Program {
  parse_program(&mut lex(source).peekable())
}
