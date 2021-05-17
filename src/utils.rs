/*

Different utility functions.

*/

use clap::{Arg, App};

use codegen::{Arch, ArchSize, AsmSyntax, CPUType, Endianess};

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

pub fn is_hex(s: &String) -> bool {
  if s.len() > 2 {
    let bytes = s.as_bytes();
    if bytes[0] == b'0' && (bytes[1] == b'x' || bytes[1] == b'X') {
      return s[2..].chars().all(|x| x.is_ascii_hexdigit());
    }
  }
  return s.chars().all(|x| x.is_ascii_hexdigit());
}

/*
  Split the .rop file from the given binaries.
*/
fn split_src_bin(mut srcs : Vec<String>) -> (String, Vec<String>) {
  let mut src : String = String::new();
  let mut cnt = 0;

  for s in &srcs {
    if s.ends_with(".rop") {
      src = s.clone();
      cnt = cnt + 1;
    }
  }
  if cnt != 1 {
    error!("expected exactly 1 source file with extension `.rop`, but got ", cnt, " instead: ",
           format!("{:?}", srcs));
  }
  else if srcs.len() < 2 {
    error!("expected at least 1 binary file, but got 0 instead: ", format!("{:?}", srcs));
  }
  srcs.retain(|x| !x.ends_with(".rop"));
  return (src.clone(), srcs);
}

pub fn parse_cmd_args() -> (String, Vec<String>, Arch, ArchSize, Endianess, AsmSyntax,
                            CPUType, Option<String>, bool, bool, bool) {
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
           .possible_values(&["arm", "mips", "sparc", "wasm", "x86"])
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
           .possible_values(&["v8", "cortex", "mips32", "mips64", "micro", "r6", "v3",
                              "v2", "v9"])
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
      .arg(Arg::with_name("pp-ast")
           .short("P")
           .long("ppast")
           .takes_value(false)
           .help("Pretty-print the abstract syntax tree.")
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
           .map(|s| s.to_string())
           .collect();

  let arch : Arch =
    match matches.value_of("architecture") {
      Some("arm")   => ArchARM,
      Some("mips")  => ArchMIPS,
      Some("sparc") => ArchSPARC,
      Some("wasm")  => ArchWasm,
      Some("x86")   => ArchX86,
      _             => error!("The instruction set architecture were not specified."),
    };

  let archsize : ArchSize =
    match matches.value_of("architecture-size") {
      Some("8")  => BitSize8,
      Some("16") => BitSize16,
      Some("32") => BitSize32,
      Some("64") => BitSize64,
      _          => error!("The instruction set architecture bit-size were not specified."),
    };

  let endianess : Endianess =
    match matches.value_of("endianess") {
      Some("lsb") => Lsb,
      Some("msb") => Msb,
      _           => error!("Endianess of the given binaries were not specified."),
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
      Some("mips32") => MIPS32,
      Some("mips64") => MIPS64,
      Some("micro")  => Micro,
      Some("r6")     => R6,
      Some("v3")     => V3,
      Some("v2")     => V2,
      Some("v9")     => V9,
      _              => NoCPUType,
    };

  let del_bytes : Option<String> =
    match matches.value_of("delete-bytes") {
      Some(s) => Some(s.to_string()),
      None    => None,
    };

  if let Some(ref del_bytes) = del_bytes {
    if !is_hex(&del_bytes) {
        error!("-d --delete \'", del_bytes, "\' is not a hexadecimal value.");
    }
  }

  let hex       : bool = matches.is_present("insert-hex-sequences");
  let ppast     : bool = matches.is_present("pp-ast");
  let ppgadgets : bool = matches.is_present("pp-gadgets");

  let (src, bins) = split_src_bin(files);

  return (src, bins, arch, archsize, endianess, syntax,
          cputype, del_bytes, hex, ppast, ppgadgets);

}
