use ast::*;

use self::Variable::*;
use self::Const::*;
use self::Exp::*;
use self::Type::*;
use self::AST::*;

macro_rules! seps {
  () => {
    ['-', '+', '/', '*', '>', '<', '=', '|', '&', '!', '%', '(', ')',
    '{', '}', ',', '.', ':', ';', '#', '@', '?', '\'', '\"', ' ']
  };
}

fn get_consts(e : &Exp) -> Option<&[Const]> {
  match e {
    Array(v)    => Some(v),
    Constant(c) => Some(std::slice::from_ref(c)),
     _          => None,
  }
}

fn get_asms<'a>(id  : &'a String,
                ast : &'a [AST]) -> Option<&'a [Const]> {
  for x in ast.iter().rev() {
    if let AST::Stat(Exp::Let(Variable::Var(vname, _, _), e)) = x {
      if vname == id {
        match get_consts(e) {
          Some(v) => return Some(v),
          None    => return Some(&[]),
        }
      }
    }
  }

  None
}

fn unfold_asm(cs : Vec<Const>) -> Vec<String> {
  let mut v = Vec::new();
  for c in cs {
    match c {
      Asm(asm, _, _) => v.push(asm),
    }
  }
  v
}

fn refs(asm  : &str,
        refs : Vec<usize>,
        i    : usize,
        ast  : &[AST]) -> (Vec<Vec<String>>, Vec<String>) {
  let mut rvals  = Vec::new();
  let mut rnames = Vec::new();
    for ridx in refs {
      let rname = asm[ridx+1..].chars()
                               .take_while(|&ch| !seps!().contains(&ch))
                               .collect::<String>();
      if let Some(arr) = get_asms(&rname, &ast[..i]) {
        rvals.push(unfold_asm(own!(arr)));
        rnames.push(own!("@") + &rname);
      }
    }
  (rvals, rnames)
}

fn perm<T : Clone>(arrays : Vec<Vec<T>>) -> Vec<Vec<T>> {
  arrays.iter().fold(vec![vec![]], |acc, arr| {
    acc.iter().flat_map(|a| arr.iter().map(move |x| {
      let mut res = a.clone();
      res.push(x.clone());
      res
    })).collect()
  })
}

fn list(list : &mut [Const],
        i    : usize,
        ast  : &[AST]) -> Vec<Const> {
  let mut v = Vec::new();
  for e in list {
    match e {
      Asm(ref mut asm, pos, AsmType) => {
        let refers : Vec<_> = asm.match_indices('@').map(|(x, _)| x).collect();
        if refers.is_empty() {
          v.push(e.clone());
          continue;
        }

        let (rvals, rnames) = refs(asm, refers, i, ast);

        let perms = perm(rvals.clone());

        for perm in perms {
          let mut asm1 = asm.clone();
          for (a, rname) in perm.iter().zip(rnames.iter()) {
            asm1 = asm1.replacen(rname, a, 1);
          }
          v.push(Asm(string!(asm1), *pos, AsmType));
        }
      }
      x => v.push(x.clone()),
    }
  }

  v
}

fn gadget(gadget : &mut [Const],
          i      : usize,
          ast    : &mut [AST]) {
  let v = list(gadget, i, ast);
    match &mut ast[i] {
      Stat(Gadget(g))            => *g = v,
      Stat(Let(Var(_, _, _), g)) => *g = box_!(Gadget(v)),
      x                          => i_e!("not a Gadget: ", pp!(x)),
    }
}

fn const_(c   : &Const,
          i   : usize,
          ast : &mut [AST]) {
  match c {
    Asm(asm, pos, ty)  => {
      let refers : Vec<_> = asm.match_indices('@').map(|(x, _)| x).collect();
      if refers.is_empty() { return (); }

      let (rvals, rnames) = refs(asm, refers, i, ast);

      let     perms = perm(rvals.clone());
      let mut res   = Vec::new();

      for perm in perms {
        let mut asm1 = asm.clone();
        for (a, rname) in perm.iter().zip(rnames.iter()) {
          asm1 = asm1.replacen(rname, a, 1);
        }
        res.push(asm1);
      }

      let v = res.iter().cloned().collect::<String>();
      match &mut ast[i] {
        Stat(Constant(Asm(a, _, _))) => *a = v,
        Stat(Let(Var(_, _, _), c))   => *c = box_!(Constant(Asm(v, *pos, *ty))),
        x                            => i_e!("not a Const: ", pp!(x)),
      }
    }
  }
}

fn array(arr : &mut [Const],
         i   : usize,
         ast : &mut [AST]) {
  let v = list(arr, i, ast);
  match &mut ast[i] {
    Stat(Let(Var(_, _, _), a)) => *a = box_!(Array(v)),
    x                          => i_e!("not an Array: ", pp!(x)),
  }
}

fn call(_var : &Variable) {
  ()
}

fn let_(e   : &mut Exp,
        i   : usize,
        ast : &mut [AST]) {
  match e {
    Gadget(g)   => gadget(g, i, ast),
    Call(v)     => call(v),
    Array(a)    => array(a, i, ast),
    Constant(c) => const_(c, i, ast),
    _           => i_e!("unexpected construct: ", pp!(e)),
  };
}

pub fn ir(ast : &mut [AST]) {
  let mut ast_cp : Vec<AST> = vec![Stat(Empty); ast.len()];
  ast_cp.clone_from_slice(ast);
  ast_cp.iter_mut().enumerate().for_each(|(i, x)| {
    match x {
      Stat(Gadget(g)) => gadget(g, i, ast),
      Stat(Let(_, e)) => let_(e, i, ast),
      _               => (),
    }
  });
}
