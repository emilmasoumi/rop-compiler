use std::iter::Peekable;
use std::str::Chars;

use utils::highlight;
use ast::*;
use typechecker::get_type;

use self::Pos::Pos;
use self::Variable::*;
use self::Const::*;
use self::Exp::*;
use self::Type::*;
use self::AST::*;

pub struct Token<'a> {
  pub col  : usize,
  pub line : usize,
  pub src  : &'a String,
  pub xs   : &'a mut Peekable<Chars<'a>>,
}

macro_rules! line { ($t:ident) => { $t.line }; }
macro_rules! col  { ($t:ident) => { $t.col  }; }
macro_rules! pos  { ($t:ident) => { Pos($t.line, $t.col)  }; }

macro_rules! sym      { ($t:ident) => { sym($t.xs) }; }
macro_rules! next_sym { ($t:ident) => {{ align($t); next_sym($t.xs) }} }

macro_rules! p_e {
  ($t:ident, $exp:expr, $($args:tt)*) => {
    error![$($args)*, "\n", highlight($t.src, &$exp, Pos($t.line, $t.col))]
  };
}

macro_rules! i_e {
  ($t:ident, $exp:expr, $($args:tt)*) => {
    p_e!($t, $exp, "internal: ", $($args)*)
  };
}

#[inline(always)]
fn some_eq<T : PartialEq>(xs : &[T], e : &T) -> bool {
  xs.iter().find(|&x| x == e) != None
}

#[inline(always)]
fn sep(c : char) -> bool {
  some_eq(&[',', ';', '=', '\"', '/', '[', ']', '{', '}', '\0'], &c) || is_whitespace!(c)
}

#[inline(always)]
fn sep_str(xs : &str) -> bool {
  some_eq(&[",", ";", "=", "\"", "/", "[", "]", "{", "}", "\0"], &xs) || xs.trim().is_empty()
}

#[inline(always)]
fn reserved(xs : &str) -> (char, bool) {
  let ys = ['-', '+', '/', '*', '>', '<', '=', '|', '&', '!', '%', '(', ')',
            '{', '}', ',', '.', ':', ';', '#', '@', '?', '\'', '\"'];
  for y in ys { if xs.contains(y) { return (y, true); } }
  (' ', false)
}

#[inline(always)]
fn is_string(xs : &str) -> bool {
  xs.starts_with('\"') && xs.ends_with('\"')
}

#[inline(always)]
fn align(t : &mut Token<'_>) {
  if sym!(t) == '\n' {
    line!(t) = line!(t) + 1;
    col!(t)  = 0;
  }
  col!(t) = col!(t) + 1;
}

#[inline(always)]
fn sym(xs : &mut Peekable<Chars<'_>>) -> char {
  match xs.peek() {
    Some(&x) => x,
    _        => '\0',
  }
}

#[inline(always)]
fn next_sym(xs : &mut Peekable<Chars<'_>>) -> char {
  let s = sym(xs);
  xs.next();
  s
}

fn backslash(t : &mut Token<'_>) -> String {
  let mut s = String::new();
  while sym!(t) == '\\'  { s.push(next_sym!(t)); }
  if (s.len() % 2) != 0  { s.push(next_sym!(t)); }
  s
}

fn string(t : &mut Token<'_>) -> String {
  let mut s = string!(next_sym!(t));
  while sym!(t) != '\"' {
    match sym!(t) {
      '\\' => s.push_str(&backslash(t)),
      '\0' => p_e!(t, s, "expected closing `\"` "),
      _    => s.push(next_sym!(t)),
    }
  }
  s.push(next_sym!(t));
  if s.len() < 3 { p_e!(t, s, "some assembly is expected in: `\"\"`"); }
  s
}

fn multiline_comment(t : &mut Token<'_>) {
  let mut s : char;
  next_sym!(t);
  s = sym!(t);
  while s != '\0' {
    next_sym!(t);
    if s == '*' && sym!(t) == '/' {
      next_sym!(t);
      return;
    }
    s = sym!(t);
  }
  p_e!(t, string!(""), "expected `*\\` somewhere after this: `/*`: ");
}

fn comment(t : &mut Token<'_>) {
  next_sym!(t);
  match sym!(t) {
    '/' => while !some_eq(&['\n', '\0'], &next_sym!(t)) {},
    '*' => multiline_comment(t),
    _   => p_e!(t, string!(sym!(t)), "expected `/` or `*`"),
  };
}

fn accept(t : &mut Token<'_>) -> String {
  match sym!(t) {
    '/'  => { comment(t); return accept(t); }
    '\"' => return string(t),
    _    => ()
  };

  if is_whitespace!(sym!(t)) {
    next_sym!(t);
    return accept(t);
  }
  else if sep(sym!(t)) {
    return string!(next_sym!(t));
  }

  let mut s = string!(next_sym!(t));
  while !sep(sym!(t)) { s.push(next_sym!(t)); }
  s
}

fn unstring(xs : &str) -> &str {
  let mut chars = xs.chars();
  chars.next();
  chars.next_back();
  chars.as_str()
}

fn semicolon(t  : &mut Token<'_>,
             xs : String) {
  if xs != ";" {
    let s = accept(t);
    if s != ";" { p_e!(t, s, "expected: `;`, but got: ", s) }
  }
}

fn asm(t   : &mut Token<'_>,
       arr : &mut Vec<Const>,
       ast : &mut [AST]) {
  let xs = accept(t);
  if is_string(&xs) { arr.push(Asm(string!(unstring(&xs)), pos!(t), AsmType)) }
  else {
    let x = lookup(&xs, ast);
    match x {
      Some(Stat(Let(Var(id, _, GadgetType), _))) =>
        arr.push(Asm(string!(id), pos!(t), GadgetType)),
      Some(Stat(Let(Var(_, _, AsmType), e))) =>
        match &**e {
          Constant(Asm(s, _, _)) => arr.push(Asm(string!(s), pos!(t), AsmType)),
          y                      => p_e!(t, string!(xs), "unexpected const construct: `",
                                         xs, "` of form: `", pp!(y), "`"),
        },
      Some(Stat(Let(_, e))) =>
        match &**e {
          Array(a)  => arr.append(&mut a.to_vec()),
          y         => p_e!(t, string!(xs), "unexpected construct: `", xs,
                            "` of form: `", pp!(y), "`"),
        },
      None => p_e!(t, string!(xs), "undefined identifier referenced: ", xs),
      y    => i_e!(t, string!(xs), "unmatched: `", xs, "`: `", pp!(y), "`"),
    }
  }
}

fn set(t       : &mut Token<'_>,
       ast     : &mut [AST],
       closing : String) -> Exp {
  let arr = &mut Vec::new();
  asm(t, arr, ast);
  let mut xs = accept(t);
  while xs != closing {
    match as_str!(xs) {
      "," => asm(t, arr, ast),
      _   => p_e!(t, string!(xs), "unexpected symbol: `", xs,
                  "` expected `,` or `", closing, "`"),
    };
    xs = accept(t);
  }
  if arr.is_empty() { p_e!(t, string!(""), "empty sets are disallowed") }
  match as_str!(closing) {
    "]" => Array(to_vec!(arr)),
    "}" => Gadget(to_vec!(arr)),
    _   => error!("set operation: unexpected symbol: ", closing),
  }
}

fn gadget(t   : &mut Token<'_>,
          ast : &mut [AST]) -> Exp {
  set(t, ast, string!("}"))
}

fn array(t   : &mut Token<'_>,
         ast : &mut [AST]) -> Exp {
  set(t, ast, string!("]"))
}

fn const_(t  : &mut Token<'_>,
          xs : String) -> Exp {
  if is_string(&xs) { Constant(Asm(string!(unstring(&xs)), pos!(t), AsmType)) }
  else { p_e!(t, string!(xs), "expected some assembly, but got: ", xs) }
}

fn ref_(t   : &mut Token<'_>,
        id  : String,
        ast : &mut [AST]) -> Exp {
  match lookup(&id, ast) {
    Some(Stat(Let(_, e))) => *e.clone(),
    _ => p_e!(t, string!(id), "undefined identifier referenced: ", id)
  }
}

fn call(t   : &mut Token<'_>,
        id  : String,
        ast : &mut [AST]) -> Exp {
  match lookup(&id, ast) {
    Some(Stat(Let(Var(_, _, ty), _))) => Call(Var(id, pos!(t), *ty)),
    _                                 =>
      p_e!(t, string!(id), "undefined identifier called: ", id)
  }
}

fn let_(t   : &mut Token<'_>,
        id  : String,
        ast : &mut [AST]) -> Exp {
  let xs = accept(t);
  match xs.chars().next().unwrap() {
    '{' => Let(Var(id, pos!(t), GadgetType), box_!(gadget(t, ast))),
    '"' => Let(Var(id, pos!(t), AsmType),    box_!(const_(t, xs))),
    '[' => Let(Var(id, pos!(t), ArrayType),  box_!(array(t, ast))),
    _   => {
      let ty = get_type(&xs, ast);
      match ty {
        GadgetType => Let(Var(string!(id), pos!(t), ty), box_!(call(t, xs, ast))),
        _          => Let(Var(string!(id), pos!(t), ty), box_!(ref_(t, xs, ast))),
      }
    }
  }
}

fn ident(t   : &mut Token<'_>,
         id  : String,
         ast : &mut [AST]) -> Exp {
  let ys = accept(t);
  match as_str!(ys) {
    "=" => let_(t, id, ast),
    ";" => call(t, id, ast),
    _   => p_e!(t, ys, "unexpected identifier: `", ys, "`"),
  }
}

fn exp(t   : &mut Token<'_>,
       xs  : &str,
       ast : &mut [AST]) -> Exp {
  match xs {
    "{" => gadget(t, ast),
    ";" => Empty,
    _   => {
      let (sym, is_sym) = reserved(xs);
      if sep_str(xs)
        { p_e!(t, string!(xs), "unexpected keyword: `", xs, "`") }
      else if is_sym
        { p_e!(t, string!(xs),  "the symbol: `", sym, "` in `", xs, "` is reserved") }
      else
        { ident(t, string!(xs), ast) }
    }
  }
}

fn statement(t   : &mut Token<'_>,
             xs  : &str,
             ast : &mut Vec<AST>) {
  let expr = exp(t, xs, ast);
  match expr {
    Call(_) => (),
    _       => semicolon(t, string!(xs)),
  }
  if expr != Empty { ast.push(Stat(expr)) };
}

fn block(t   : &mut Token<'_>,
         ast : &mut Vec<AST>) {
  let mut xs = accept(t);
  while xs != "\0" {
    statement(t, &xs, ast);
    xs = accept(t);
  }
}

pub fn parser(src : &String) -> Vec<AST> {
  let mut ast : Vec<AST> = Vec::new();
  let mut it = src.chars().peekable();
  let mut t  = Token { line: 1, col: 1, src: src, xs: &mut it };
  block(&mut t, &mut ast);
  ast
}
