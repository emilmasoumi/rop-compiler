use clap::{Arg, App};

use ast::{Pos};
use codegen::{Arch, ArchSize, AsmSyntax, CPUType, Endianess};

use std::io::prelude::*;
use std::fs::File;

use std::fmt::Debug;

use self::Pos::*;
use self::Arch::*;
use self::ArchSize::*;
use self::AsmSyntax::*;
use self::CPUType::*;
use self::Endianess::*;

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
macro_rules! is_whitespace { ($exp:expr) => { $exp.is_whitespace() }; }

macro_rules! as_str { ($id:ident) => { $id.as_str() }; }
macro_rules! pp     { ($id:ident) => { format!("{:?}", $id) }; }

macro_rules! box_v { ($e:expr) => { Box::new($e.to_vec()) }; }
macro_rules! box_  { ($e:expr) => { Box::new($e) }; }

macro_rules! to_vec { ($e:expr) => { $e.to_vec() }; }

pub fn highlight(src : &String, id: &String, pos : Pos) -> String {
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

pub fn is_hex(s : &String) -> bool {
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

fn get_src_bin(mut srcs : Vec<String>) -> (String, Vec<String>) {
  let mut src : String = String::new();

  let cnt = srcs.iter().fold(0, |acc, s|
      if s.ends_with(".rop") {src = s.clone(); acc + 1} else {acc});

  if cnt != 1 {
    error!("expected exactly 1 source file with extension `.rop`, but got ",
           cnt, " instead: ", pp!(srcs));
  }
  else if srcs.len() < 2 {
    error!("expected at least 1 binary file, but got 0 instead: ", pp!(srcs));
  }

  srcs.retain(|x| !x.ends_with(".rop"));
  (src, srcs)
}

pub fn parse_cmd_args() -> (String, Vec<String>, Arch, ArchSize, Endianess,
                            AsmSyntax, CPUType, Option<String>, bool, bool) {
  let matches =
      App::new("ROP and JOP compiler")
      .version("0.0.1")
      .about("Return-oriented programming and jump-oriented programming compiler.")
      /* arguments: */
      .arg(Arg::with_name("binaries")
           .required(true)
           .multiple(true)
           .help("Binary file(s)")
          )
      .arg(Arg::with_name("architecture")
           .short("a")
           .long("arch")
           .required(true)
           .takes_value(true)
           .possible_values(&["arm", "x86"])
           .help("Instruction set architecture.")
          )
      .arg(Arg::with_name("architecture-size")
           .short("b")
           .long("archsize")
           .required(true)
           .takes_value(true)
           .possible_values(&["8", "16", "32", "64"])
           .help("Architecture bit-size.")
          )
      .arg(Arg::with_name("endianess")
           .short("e")
           .long("endianess")
           .required(true)
           .takes_value(true)
           .possible_values(&["lsb", "msb"])
           .help("Specifies the endianess of the binaries.")
          )
      .arg(Arg::with_name("assembly-syntax")
           .short("s")
           .long("select")
           .required(true)
           .takes_value(true)
           .possible_values(&["att", "intel"])
           .help("Select the assembly syntax.")
          )
      /* options: */
      .arg(Arg::with_name("cpu-type")
           .short("c")
           .long("cpu")
           .takes_value(true)
           .possible_values(&["v8", "cortex", "r6", "v3", "v2", "v9"])
           .help("Select a specific CPU type.")
          )
      .arg(Arg::with_name("delete-bytes")
           .short("d")
           .long("delete")
           .takes_value(true)
           .help("Delete all occurrences of the given byte from the payload.")
          )
      .arg(Arg::with_name("insert-hex-sequences")
           .short("H")
           .long("hex")
           .takes_value(false)
           .help("Insert hexadecimal escape sequences for every byte in the payload.")
          )
      .arg(Arg::with_name("pp-gadgets")
           .short("p")
           .long("ppgadgets")
           .takes_value(false)
           .help("Pretty-print the gadget chain.")
          )
      .get_matches();

  let files : Vec<_> =
    matches.values_of("binaries")
           .unwrap()
           .map(|s| string!(s))
           .collect();

  let arch : Arch =
    match matches.value_of("architecture") {
      Some("arm")   => ArchARM,
      Some("x86")   => ArchX86,
      _             => u_e!("The instruction set architecture were not specified."),
    };

  let archsize : ArchSize =
    match matches.value_of("architecture-size") {
      Some("8")  => BitSize8,
      Some("16") => BitSize16,
      Some("32") => BitSize32,
      Some("64") => BitSize64,
      _          => u_e!("The instruction set architecture bit-size were not specified."),
    };

  let endianess : Endianess =
    match matches.value_of("endianess") {
      Some("lsb") => Lsb,
      Some("msb") => Msb,
      _           => u_e!("Endianess of the given binaries were not specified."),
    };

  let syntax : AsmSyntax =
    match matches.value_of("assembly-syntax") {
      Some("intel") => Intel,
      _             => Att,
    };

  let cputype : CPUType =
    match matches.value_of("cpu-type") {
      Some("v8")     => V8,
      Some("cortex") => Cortex,
      Some("r6")     => R6,
      Some("v3")     => V3,
      Some("v2")     => V2,
      Some("v9")     => V9,
      _              => NoCPUType,
    };

  let del_bytes : Option<String> =
    matches.value_of("delete-bytes").map(|s| string!(s));

  if let Some(ref del_bytes) = del_bytes {
    if !is_hex(del_bytes) {
        error!("-d --delete \'", del_bytes, "\' is not a hexadecimal value.");
    }
  }

  let hex       : bool = matches.is_present("insert-hex-sequences");
  let ppgadgets : bool = matches.is_present("pp-gadgets");

  let (src, bins) = get_src_bin(files);

  (src, bins, arch, archsize, endianess, syntax,
   cputype, del_bytes, hex, ppgadgets)

}
