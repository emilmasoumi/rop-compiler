/*

Parse the stream of tokens into an abstract syntax tree.

*/

use utils::{err_line, read_file, pp_vec};
use lexer::Token;
use lexer::Token::*;
use lexer::Pos;

use symtab::*;
use ast::*;
use self::SymTab::*;
use self::Name::*;
use self::Exp::*;
use self::Type::*;
use self::AST::*;
use self::Pos::*;
use self::IR::*;

/* Intermediate representation of the parse tree to concatenate unseparated tokens.  */
#[derive(Debug, Clone)]
enum IR {
  EntryIR,
  BlockStartIR  (Pos),
  BlockEndIR    (Pos),
  CommaIR       (Pos),
  EqualIR       (Pos),
  IdentifierIR  (Pos, Id),
  StringStartIR (Pos),
  StringEndIR   (Pos),
  SemicolonIR   (Pos),
  AssemblyIR    (Pos, Id),
  WhitespaceIR  (Pos),
}

fn is_sep(t: &IR) -> bool {
  match t {
    BlockStartIR  (_) |
    BlockEndIR    (_) |
    CommaIR       (_) |
    EqualIR       (_) |
    StringStartIR (_) |
    StringEndIR   (_) |
    SemicolonIR   (_) |
    WhitespaceIR  (_) => true,
    _                 => false,
  }
}

fn ir_to_string(ir : &IR) -> String {
  match ir {
    EntryIR               => "entry",
    BlockStartIR  (_)     => "{",
    BlockEndIR    (_)     => "}",
    CommaIR       (_)     => ",",
    EqualIR       (_)     => "=",
    IdentifierIR  (_, id) => id,
    StringStartIR (_)     => "\"",
    StringEndIR   (_)     => "\"",
    SemicolonIR   (_)     => ";",
    AssemblyIR    (_, id) => id,
    WhitespaceIR  (_)     => " ",
    _                     => error!("ir_to_string(): given IR was not found: ",
                                    format!("{:?}", ir)),
  }.to_string()
}

fn tok_to_ir(t : Token, ir : IR) -> IR {
  match (t, ir) {
    (Identifier(_, c), IdentifierIR(p, s)) => {
      let id = s + &c.to_string();
      return IdentifierIR(p, id);
    }
    (AssemblyTok(_, c), AssemblyIR(p, s)) => {
      let id = s + &c.to_string();
      return AssemblyIR(p, id);
    }
    (BlockStart(p), _) => {
      return BlockStartIR(p);
    }
    (BlockEnd(p), _) => {
      return BlockEndIR(p);
    }
    (Comma(p), _) => {
      return CommaIR(p);
    }
    (EqualTok(p), _) => {
      return EqualIR(p);
    }
    (Identifier(p, c), _) => {
      return IdentifierIR(p, c.to_string());
    }
    (StringStart(p), _) => {
      return StringStartIR(p);
    }
    (StringEnd(p), _) => {
      return StringEndIR(p);
    }
    (Semicolon(p), _) => {
      return SemicolonIR(p);
    }
    (AssemblyTok(p, c), _) => {
      return AssemblyIR(p, c.to_string());
    }
    (Whitespace(p), _) => {
      return WhitespaceIR(p);
    }
    (x, y) => {
      error!("tok_to_ir(): unexpected match:\n", format!("{:?}", x),
             "\nwith\n", format!("{:?}", y));
    }
  }
}

/* Remove whitespaces from the intermediate representation of the parse tree. */
fn remove_ws_ir(ir : Vec<IR>) -> Vec<IR> {
  let mut irs : Vec<IR> = Vec::new();
  for n in ir {
    match n {
      WhitespaceIR(_) => {}
      a               => { irs.push(a); }
    }
  }
  return irs;
}

/* Parse the parse tree to its intermediate representation */
fn parse_ir(parse_tree : &Vec<Token>) -> Vec<IR> {
  let mut ir : Vec<IR> = Vec::new();

  ir.push(EntryIR);
  for tok in parse_tree {
    let idx = ir.len()-1;
    let n   = tok_to_ir(tok.clone(), ir[idx].clone());
    if is_sep(&n) || is_sep(&ir[idx]) {
      ir.push(n);
    } else {
      ir[idx] = n;
    }
  }

  let ir = remove_ws_ir(ir);
  return ir;
}

fn get_type(symtab : Vec<SymTab>, s : Id) -> Type {
  match lookup(symtab, s) {
    Some(Entry(id, ty)) => ty,
    None                => UndefType,
  }
}

/* Parse the intermediate representation of the parse tree to an abstract syntax tree. */
pub fn parser(parse_tree : &Vec<Token>, src : String) -> Vec<AST> {
  let mut symtab : Vec<SymTab> = Vec::new();
  let mut ast    : Vec<AST>    = Vec::new();

  let ir = parse_ir(parse_tree);

  let mut it = ir.iter().peekable();
  while let Some(&n) = it.peek() {
    match n {
      IdentifierIR(p, s) => {
        match &s[..] {
          "let" => {
            //let v = Let(...)
          }
          _     => {
            let ty = get_type(symtab.clone(), s.to_string());
            it.next();
            match it.peek() {
              Some(EqualIR(pos)) => {

              }
              Some(SemicolonIR(pos)) => {
                match ty {
                  UndefType => {
                    error!("reference to undefined identifier: \'", s, "\'\n",
                           err_line(&src, s, p));
                  }
                  _ => {}
                }
              }
              Some(CommaIR(pos)) => {

              }
              _ => {}
            }
            symtab.push(Entry(s.clone(), UndefType));
            let v = Ref(Var(s.clone(), UndefType));
            ast.push(Stat(v));
          }
        }
      }
      BlockStartIR(p) => {
        // ...
      }
      _ => {}
    }

    it.next();
  }
  return ast;
}
