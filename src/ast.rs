/*

The abstract syntax tree.

*/

pub type Id = String;

#[derive(Debug, Clone)]
pub enum Type {
  ArrayType,
  AssemblyType,
  BlockType,
  UndefType,
}

#[derive(Debug)]
pub enum Operator {
  Equal,
}

#[derive(Debug)]
pub enum Name {
  Var (Id, Type),
}

#[derive(Debug)]
pub enum Exp<'a, 'b> {
  Assign   (Name, &'a Exp<'a, 'b>),
  Block    (Type, &'a Exp<'a, 'b>),
  Variable (Name),
  Let      (&'a Exp<'a, 'b>),
  Assembly (Id),
  Array    (&'b [Exp<'a, 'b>], Type),
  Op       (Operator),
  Lambda   (&'a &'b Exp<'a, 'b>, Type),
  Ref      (Name),
  Empty,
}

#[derive(Debug)]
pub enum AST<'a, 'b> {
  Stat (Exp<'a, 'b>),
}

pub fn pp_ast(ast : Vec<AST>) -> () {
  println!("-------- Printing the abstract syntax tree --------");
  for node in ast {
    println!("{:?}", node);
  }
}
