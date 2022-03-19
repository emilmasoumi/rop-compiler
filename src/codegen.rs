use capstone::prelude::*;

use std::process::Command;
use std::io::{self, Write};

use ast::*;
use utils::{is_hex};

use self::Arch::*;
use self::ArchSize::*;
use self::AsmSyntax::*;
use self::CPUType::*;
use self::Endianess::*;

#[derive(Debug)]
pub enum Arch {
  ArchARM,
  ArchX86,
}

#[derive(Debug)]
pub enum ArchSize {
  BitSize8,
  BitSize16,
  BitSize32,
  BitSize64,
}

#[derive(Debug)]
pub enum AsmSyntax {
  Att,
  Intel,
}

// rasm2 CPU types
#[derive(Debug)]
pub enum CPUType {
  NoCPUType,
  V8,
  Cortex,
  R6,
  V3,
  V2,
  V9,
}

#[derive(Debug)]
pub enum Endianess {
  Lsb,
  Msb,
}

static ARCHS:     &[&str] = &["arm", "x86"];
static ARCHSIZES: &[&str] = &["8", "16", "32", "64"];
static CPUTYPES:  &[&str] = &["", "v8", "cortex", "r6", "v3", "v2", "v9"];
static ENDIAN:    &[&str] = &["", "-e"];
static SYNTAX:    &[&str] = &["att", "intel"];

fn transform() {
}

fn assemble(code : String, arch_idx : usize, archsize_idx : usize,
            endianess_idx : usize, cputype_idx : usize, syntax_idx : usize) -> String {
  let mut options = vec![];
  let arch        = ARCHS[arch_idx];
  let archsize    = ARCHSIZES[archsize_idx];
  let cputype     = CPUTYPES[cputype_idx];
  let endianess   = ENDIAN[endianess_idx];
  let syntax      = SYNTAX[syntax_idx];

  options.push("-a");
  options.push(arch);
  options.push("-b");
  options.push(archsize);
  options.push("-s");
  options.push(syntax);
  if !cputype.is_empty() {
    options.push("-c");
    options.push(cputype);
  }
  if !endianess.is_empty() {
    options.push(endianess);
  }
  options.push(&code);

  let opcodes = if cfg!(target_os = "windows") {
    Command::new("start C:\rasm2.exe")
            .args(&options)
            .output()
            .expect("failed executing a process")
  } else {
    Command::new("/bin/rasm2")
            .args(&options)
            .output()
            .expect("failed executing a process")
  };

  let failure     = String::from_utf8_lossy(&opcodes.stderr);
  let opcodes_str = String::from_utf8_lossy(&opcodes.stdout).split_whitespace().collect();

  if failure.len() > 0 || !opcodes.status.success() {
    let sh_cmd = if cfg!(target_os = "windows") {
      "start C:\rasm2.exe "
    }
    else {
      "/bin/rasm2 "
    };
    error!("assembling:\n", code, "\nfailed with output:\n", failure, opcodes_str,
           "\nfrom the following shell command:\n", sh_cmd, options.join(" "));
  }

  if !is_hex(&opcodes_str) {
    error!("the opcodes:\n", opcodes_str, "\nreturned from assembling:\n", code,
           "\nis not of hexadecimal format.");
  }

  opcodes_str
}

fn read_bytes(bin : &str) -> Vec<u8> {
  match std::fs::read(bin) {
    Ok(bytes) => {
      bytes
    }
    Err(e) => {
      error!("failed reading from: \'", bin, "\': ", e);
    }
  }
}

fn byte_to_nibbles(byte : u8) -> (u8, u8) {
  ((byte >> 4) & 0b1111, byte & 0b1111)
}

pub fn codegen(ast : &[AST], bins : Vec<String>, arch : Arch,
               archsize : ArchSize, endianess : Endianess, syntax : AsmSyntax,
               cputype : CPUType) -> String {
  codegen_(ast, bins, arch as usize, archsize as usize, endianess as usize,
           syntax as usize, cputype as usize)
}

/* Dummy code for now. */
fn codegen_(ast : &[AST], bins : Vec<String>, arch : usize, archsize : usize,
            endianess : usize, syntax : usize, cputype : usize) -> String {
  assemble("mov rax, rax; mov rax, rax;".to_string(),
           arch , archsize, endianess, cputype, syntax);
//  println!("{}", assemble("mov rax, rax; mov rax, rax;".to_string(),
//           arch , archsize, endianess, cputype, syntax));
//  println!("{}", assemble("add rsi, rcx;".to_string(),
//           arch , archsize, endianess, cputype, syntax));

  // Only the main program is considered for now.
  let prog_opcodes = read_bytes(&bins[0]);

  // Find memory addresses of the instruction locations in the given `bins` for each byte or
  // nibble in a sequence of opcodes.

  let mut cnt = 0;
  for x in &prog_opcodes {
    let (nib1, nib2) = byte_to_nibbles(*x);
    if nib1 == 0xc && nib2 == 0x3 {
      cnt += 1;
    }
  }
  //println!("cnt = {}", cnt);

  string!("")
}
