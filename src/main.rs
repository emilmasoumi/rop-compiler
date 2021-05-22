/*

Invoke different compiler stages and program parameters.

*/

#![allow(dead_code)]

extern crate capstone;
extern crate clap;

#[macro_use]
mod utils;

mod ast;
mod symtab;
mod lexer;
mod parser;
mod typechecker;
mod codegen;

use codegen::{Arch, ArchSize, AsmSyntax, CPUType, Endianess};

fn main() {
  let (src, bins, arch, archsize, endianess, syntax, cputype,
       del_bytes, insert_hex, ppast, ppgadgets) = utils::parse_cmd_args();

  let src        = utils::read_file(&src);
  let parse_tree = lexer::lexer(&src);
  let ast        = parser::parser(&parse_tree, src);

  if ppast {
    utils::pp_vec(&ast);
  }

  let payload =
    codegen::codegen(bins, arch, archsize, endianess, syntax, cputype);

  if ppgadgets {
  }

  if let Some(ref del_bytes) = del_bytes {
  }

  if insert_hex {
  }

  //println!("{}", payload);

}
