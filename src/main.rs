#![allow(dead_code)]

extern crate capstone;
extern crate clap;

#[macro_use]
mod utils;

mod ast;

mod parser;
mod typechecker;
mod codegen;

use codegen::{Arch, ArchSize, AsmSyntax, CPUType, Endianess};

fn main() {
  let (src, bins, arch, archsize, endianess, syntax, cputype,
       del_bytes, insert_hex, ppgadgets) = utils::parse_cmd_args();

  let src = utils::read_file(&src);
  let ast = parser::parser(&src);
  typechecker::typechecker(&src, &ast);

  let payload =
    codegen::codegen(&ast, bins, arch, archsize, endianess, syntax, cputype);

  if ppgadgets {
  }

  if let Some(ref del_bytes) = del_bytes {
  }

  if insert_hex {
  }

  //println!("{}", payload);
}
