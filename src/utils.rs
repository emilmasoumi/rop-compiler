use ast::{Pos};

use clap::{Arg, App};
use capstone::prelude::arch::*;
use keystone_engine::*;

use std::io::prelude::*;
use std::fs::File;
use std::fmt::Debug;

use self::Pos::*;

macro_rules! error {
  ($($args:expr),*) => {{
     print!("error: ");
     $(
        print!("{}", $args);
     )*
     print!("\n");
     std::process::exit(1);
  }}
}

macro_rules! u_e { ($($args:tt)*) => { error!("cmd: ", $($args)*) }; }

macro_rules! string        { ($exp:expr) => { $exp.to_string()     }; }
macro_rules! own           { ($exp:expr) => { $exp.to_owned()      }; }
macro_rules! is_whitespace { ($exp:expr) => { $exp.is_whitespace() }; }

macro_rules! as_str { ($id:ident) => { $id.as_str() }; }
macro_rules! pp     { ($id:ident) => { format!("{:?}", $id) }; }

macro_rules! box_v { ($e:expr) => { Box::new($e.to_vec()) }; }
macro_rules! box_  { ($e:expr) => { Box::new($e) }; }

macro_rules! to_vec { ($e:expr) => { $e.to_vec() }; }

pub fn highlight(src : &str,
                 id  : &str,
                 pos : Pos) -> String {
  let (l, c) = match pos { Pos(x, y) => (x, y) };
  let split : Vec<_> = src.split('\n').collect();
  println!(" {}:{} | {}", l, c, split[l-1]);

  let ws  = 3 + string!(l).len() + string!(c).len();
  " ".repeat(ws) + "|" + &" ".repeat(c-id.len()) + &"^".repeat(id.len())
}

pub fn read_file(filename : &String) -> String {
  let fs = File::open(filename);

  let mut fs = match fs {
    Ok(file) => file,
    Err(e)   => error!("failed opening: \'", filename, "\': ", e),
  };

  let mut s = String::new();

  match fs.read_to_string(&mut s) {
    Ok(_)  => s,
    Err(e) => error!("failed reading: \'", filename, "\' into a string: ", e),
  }
}

pub fn is_hex(s : &str) -> bool {
  if s.len() > 2 {
    let bytes = s.as_bytes();
    if bytes[0] == b'0' && (bytes[1] == b'x' || bytes[1] == b'X') {
     return s[2..].chars().all(|x| x.is_ascii_hexdigit())
    }
  }
  s.chars().all(|x| x.is_ascii_hexdigit())
}

// fmt::Debug: {:?} depicts '\' as '\\\'.
pub fn pp_vec<T : Debug>(v : Vec<T>) {
  v.iter().for_each(|e| println!("{:?}", e));
}

fn get_src_bin(mut srcs : Vec<String>) -> (String, String) {
  let mut src : String = String::new();

  let cnt = srcs.iter().fold(0, |acc, s|
      if s.ends_with(".rop") {src = s.clone(); acc + 1} else {acc});

  if cnt != 1 {
    error!("expected exactly 1 source file with the extension `.rop`, but got ",
           cnt, " instead: ", pp!(srcs));
  }
  else if srcs.len() != 2 {
    error!("expected exactly 1 binary file, but got ", srcs.len()-cnt,
           " instead: ", pp!(srcs));
  }

  srcs.retain(|x| !x.ends_with(".rop"));
  (src, own!(srcs[0]))
}

pub fn parse_cmd_args() -> (String,
                            String,
                            (Arch, Mode),
                            (OptionValue, x86::ArchSyntax),
                            bool,
                            bool) {
  let matches =
      App::new("ROP and JOP compiler")
      .version("0.0.1")
      .about("A generic gadget chain compiler that supports the \
              architectures: ARM/ARM64, MIPS32/64, SPARC32/64, x86/x64, and \
              file formats: ELF, PE, Mach-O.")
      /* arguments: */
      .arg(Arg::with_name("files")
           .required(true)
           .multiple(true)
           .help("A `.rop` file and a binary executable.")
          )
      .arg(Arg::with_name("cpu-type")
           .short('c')
           .long("cputype")
           .required(true)
           .takes_value(true)
           .possible_values(&["arm", "thumb", "armv8", "micro", "mips3",
                              "mips32r6", "mips32", "mips64",
                              "sparc32", "sparc64", "sparcv9",
                              "x86-16", "x86-32", "x86-64"])
           .help("Computer architecture/CPU type.")
          )
      /* options: */
      .arg(Arg::with_name("assembly-syntax")
           .short('s')
           .long("syntax")
           .required_ifs(&[("cpu-type", "x86-16"),
                           ("cpu-type", "x86-32"),
                           ("cpu-type", "x86-64")])
           .takes_value(true)
           .possible_values(&["att", "gas", "intel", "nasm"])
           .help("The assembly syntax for dis/assembling.")
          )
      .arg(Arg::with_name("byte-wise")
           .short('b')
           .long("bytewise")
           .takes_value(false)
           .help("Search for memory addresses byte-wise instead of mnemonic-wise.")
          )
      .arg(Arg::with_name("out-ind")
           .short('o')
           .long("individually")
           .takes_value(false)
           .help("Output the addresses in the gadget chain individually.")
          )
      .get_matches();

  let files : Vec<_> =
    matches.values_of("files").unwrap().map(|s| own!(s)).collect();

  let cputype =
    match matches.value_of("cpu-type") {
      Some("arm")      => (Arch::ARM,   Mode::ARM),
      Some("thumb")    => (Arch::ARM,   Mode::THUMB),
      Some("armv8")    => (Arch::ARM,   Mode::V8),
      Some("mips2")    => (Arch::MIPS,  Mode::MICRO),
      Some("mips3")    => (Arch::MIPS,  Mode::MIPS3),
      Some("mips32r6") => (Arch::MIPS,  Mode::MIPS32R6),
      Some("mips32")   => (Arch::MIPS,  Mode::MIPS32),
      Some("mips64")   => (Arch::MIPS,  Mode::MIPS64),
      Some("sparc32")  => (Arch::SPARC, Mode::SPARC32),
      Some("sparc64")  => (Arch::SPARC, Mode::SPARC64),
      Some("sparcv9")  => (Arch::SPARC, Mode::V9),
      Some("x86-16")   => (Arch::X86,   Mode::MODE_16),
      Some("x86-32")   => (Arch::X86,   Mode::MODE_32),
      Some("x86-64")   => (Arch::X86,   Mode::MODE_64),
      _                => u_e!("The computer architecture/\
                                CPU type was not specified."),
    };

  let syntax =
    match matches.value_of("assembly-syntax") {
      Some("intel") => (OptionValue::SYNTAX_INTEL, x86::ArchSyntax::Intel),
      Some("nasm")  => (OptionValue::SYNTAX_NASM,  x86::ArchSyntax::Intel),
      Some("att")   => (OptionValue::SYNTAX_ATT,   x86::ArchSyntax::Att),
      Some("gas")   => (OptionValue::SYNTAX_GAS,   x86::ArchSyntax::Att),
      _             => (OptionValue::SYNTAX_INTEL, x86::ArchSyntax::Intel),
    };

  let bytewise = matches.is_present("byte-wise");
  let outind   = matches.is_present("out-ind");

  let (src, bin) = get_src_bin(files);

  (src, bin, cputype, syntax, bytewise, outind)
}
