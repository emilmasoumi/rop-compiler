pub type Id = String;

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Pos {
  Pos (usize, usize),
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Type {
  ArrayType,
  AsmType,
  GadgetType,
  VoidType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Variable {
  Var (Id, Pos, Type),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Const {
  Asm (String, Pos, Type),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Exp {
  Gadget   (Vec<Const>),
  Call     (Variable),
  Let      (Variable, Box<Exp>),
  Array    (Vec<Const>),
  Constant (Const),
  Empty,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AST {
  Stat (Exp),
}

pub fn lookup<'a>(id  : &String,
                  ast : &'a [AST]) -> Option<&'a AST> {
  for x in ast.iter().rev() {
    if let AST::Stat(Exp::Let(Variable::Var(vname, _, _), _)) = x {
      if vname == id { return Some(x) }
    }
  }
  None
}
