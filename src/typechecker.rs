use ast::*;
use utils::highlight;

use self::Variable::*;
use self::Const::*;
use self::Exp::*;
use self::Type::*;
use self::AST::*;

macro_rules! t_e {
  ($src:ident, $id:expr, $pos:ident, $($args:tt)*) => {
    error![$($args)*, "\n", highlight($src, &$id, *$pos)]
  };
}

fn pp_ty(ty : &Type) -> &str {
  match ty {
    ArrayType  => "type: `Array`",
    AsmType    => "type: `Asm`",
    GadgetType => "type: `Gadget`",
    VoidType   => "type: `Void`",
  }
}

fn pp_exp_ty(e : &Exp) -> &str {
  match e {
    Gadget(_)   |
    Call(_)     => "type: `Gadget`",
    Let(_, exp) => pp_exp_ty(exp),
    Array(_)    => "type: `Array`",
    Constant(_) => "type: `Asm`",
    _           => "type: unfound",
  }
}

pub fn get_type(id  : &String,
                ast : &[AST]) -> Type {
  for x in ast.iter().rev() {
    if let Stat(Let(Var(vname, _, ty), _)) = x { if vname == id { return *ty } }
  }
  VoidType
}

fn verify_ty(src : &str,
             var : &Variable,
             e   : &Exp) {
  match (var, e) {
    (Var(_, _, GadgetType), Gadget(_))   |
    (Var(_, _, GadgetType), Call(_))     |
    (Var(_, _, ArrayType),  Array(_))    |
    (Var(_, _, AsmType),    Constant(_)) => (),
    (Var(vname, pos, ty), _)             =>
      t_e!(src, vname, pos, "type mismatch: `", vname, "`: expected ",
           pp_exp_ty(e), ", actual ", pp_ty(ty)),
  }
}

fn const_(src : &str,
          c   : &Const) {
  match c {
    Asm(_, _, AsmType) => (),
    Asm(asm, pos, ty)  => t_e!(src, asm, pos, "type mismatch: `", asm,
                               "`: expected type: `Asm`, actual ", pp_ty(ty)),
  }
}

fn gadget(src    : &str,
          gadget : &[Const]) {
  for e in gadget {
    match e {
      Asm(_, _, AsmType) => (),
      Asm(asm, pos, ty)  =>
      t_e!(src, asm, pos, "type mismatch: `", asm,
           "`: expected `Asm|Array` actual ", pp_ty(ty)),
    }
  }
}

fn array(src : &str,
         arr : &[Const]) {
  for e in arr {
    match e {
      Asm(_, _, AsmType) => (),
      Asm(id, pos, ty)   =>
        t_e!(src, id, pos, "type mismatch: `", id,
             "`: expected type: `Asm|Array`, actual ", pp_ty(ty)),
    }
  }
}

fn call(src : &str,
        var : &Variable) {
  match var {
    Var(_, _, GadgetType) => (),
    Var(id, pos, ty)      => t_e!(src, own!(id)+";", pos, "type mismatch: `",
                                  id, "`: expected type: `Gadget`, actual ",
                                  pp_ty(ty), ", during call invocation"),
  }
}

fn let_(src : &str,
        var : &Variable,
        e   : &Exp) {
  match e {
    Gadget(g)   => gadget(src, g),
    Call(v)     => call(src, v),
    Array(a)    => array(src, a),
    Constant(c) => const_(src, c),
    _           => i_e!("unexpected construct: ", pp!(e)),
  };
  verify_ty(src, var, e)
}

pub fn typechecker(src : &str,
                   ast : &[AST]) {
  ast.iter().enumerate().for_each(|(_i, x)| {
    match x {
      Stat(Gadget(g)) => gadget(src, g),
      Stat(Call(v))   => call(src, v),
      Stat(Let(v, e)) => let_(src, v, e),
      _               => i_e!("wrong construct at global scope: ", pp!(x)),
    }
  });
}
