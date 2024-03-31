extern crate capstone;
extern crate clap;
extern crate keystone_engine;
extern crate object;

#[macro_use]
mod utils;

mod ast;
mod parser;
mod typechecker;
mod ir;
mod codegen;

fn main() {
  let (src, bin, cputype, syntax, bitwidth, bytewise, byteorder, outind, all) =
    utils::parse_cmd_args();

  let src     = utils::read_file(&src);
  let mut ast = parser::parser(&src);

  ir::ir(&mut ast);
  typechecker::typechecker(&src, &ast);

  let mut payload = codegen::codegen(&ast, bin, cputype, syntax, bitwidth,
                                     bytewise, byteorder, outind, all);

  if outind { payload.pop(); }

  println!("{payload}");
}
