use self::Pos::*;
use self::Exp::*;
use self::AST::*;
use self::Const::*;

use capstone::{prelude::*, Instructions};
use keystone_engine::*;
use object::{Object, ObjectSection, Endianness};

use ast::{Pos};
use ast::*;

fn read_bytes(bin : &str) -> Vec<u8> {
  match std::fs::read(bin) {
    Ok(bytes) => { bytes }
    Err(e)    => { error!("failed reading from: \'", bin, "\': ", e) }
  }
}

fn pp_pos(pos : &Pos) -> String {
  let (l, c) = match pos { Pos(x, y) => (x, y) };
  string!(l) + ":" + &string!(c)
}

fn kmp_table(needle : &[u8]) -> Vec<usize> {
  let mut table : Vec<usize> = vec![0; needle.len()];
  let mut j = 0;

  for i in 1..needle.len() {
    if needle[i] == needle[j] { table[i] = table[j]; }
    else {
      table[i] = j;
      while j > 0 && needle[i] != needle[j] { j = table[j]; }
    }
    j += 1;
  }

  table
}

fn kmp(haystack : &[u8], needle : &[u8]) -> Option<usize> {
  let (mut i, mut j, mut _idx, table) = (0, 0, 0, kmp_table(needle));
  //let mut idxs = Vec::new();

  while i < haystack.len() {
    if needle[j] == haystack[i] {
      i += 1;
      j += 1;
      if j == needle.len() {
        return Some(i-j);
        //idxs.insert(idx, i-j);
        //idx += 1;
        //j = table[j];
      }
    }
    else {
      j = table[j];
      if j == 0 {
        i += 1;
        j += 1;
      }
    }
  }

  None
  //idxs
}

fn assemble(engine : &Keystone, code : String, pos : &Pos) -> Vec<u8> {
  engine
    .asm(own!(code), 0)
    .expect(&format!("error: {} assembler: {}", pp_pos(pos), code))
    .bytes
}

fn cs(archcpu : (Arch, Mode), syntax : arch::x86::ArchSyntax) -> Capstone {
  let err_msg = format!("Failed creating the Capstone object for: {:?}", archcpu);
  // `Arch` values must be supplied in comparisons as the `Mode` values overlap.
  match archcpu {
    (Arch::ARM, Mode::ARM) => {
      Capstone::new().arm().mode(arch::arm::ArchMode::Arm).detail(true).build()
                     .expect(&err_msg)
    }
    (Arch::ARM, Mode::THUMB) => {
      Capstone::new().arm().mode(arch::arm::ArchMode::Thumb).detail(true).build()
                     .expect(&err_msg)
    }
    (Arch::MIPS, Mode::MIPS3) => {
      Capstone::new().mips().mode(arch::mips::ArchMode::Mips3).detail(true).build()
                     .expect(&err_msg)
    }
    (Arch::MIPS, Mode::MIPS32R6) => {
      Capstone::new().mips().mode(arch::mips::ArchMode::Mips32R6).detail(true).build()
                     .expect(&err_msg)
    }
    (Arch::MIPS, Mode::MIPS32) => {
      Capstone::new().mips().mode(arch::mips::ArchMode::Mips32).detail(true).build()
                     .expect(&err_msg)
    }
    (Arch::MIPS, Mode::MIPS64) => {
      Capstone::new().mips().mode(arch::mips::ArchMode::Mips64).detail(true).build()
                     .expect(&err_msg)
    }
    (Arch::SPARC, Mode::SPARC32 | Mode::SPARC64) => {
      Capstone::new().sparc().mode(arch::sparc::ArchMode::Default).detail(true).build()
                     .expect(&err_msg)
    }
    (Arch::SPARC, Mode::V9) => {
      Capstone::new().sparc().mode(arch::sparc::ArchMode::V9).detail(true).build()
                     .expect(&err_msg)
    }
    (Arch::X86, Mode::MODE_16) => {
      Capstone::new().x86().mode(arch::x86::ArchMode::Mode16).syntax(syntax)
                     .detail(true).build()
                     .expect(&err_msg)
    }
    (Arch::X86, Mode::MODE_32) => {
      Capstone::new().x86().mode(arch::x86::ArchMode::Mode32).syntax(syntax)
                     .detail(true).build()
                     .expect(&err_msg)
    }
    (Arch::X86, Mode::MODE_64) => {
      Capstone::new().x86().mode(arch::x86::ArchMode::Mode64).syntax(syntax)
                     .detail(true).build()
                     .expect(&err_msg)
    }
    (Arch::ARM, Mode::V8) | (Arch::MIPS, Mode::MICRO) => {
      /*
      Capstone::new().arm().extra_mode(arch::arm::ArchExtraMode::V8).detail(true)
      .build().expect(&err_msg)
      */
      error!("The CPU modes: V8 and Micro are not yet supported.")
    }
    a => {
      error!("Unknown CPU type given: ", format!("{:?}", a))
    }
  }
}

fn get_opcodes_addr_offs(obj : object::File<'_>) -> (&[u8], u64) {
  // We currently do not wish to consider other sections.
  let sections = [".text", "__text"];
  for s in sections {
    if let Some(section) = obj.section_by_name(s) {
      if let Ok(opcodes) = section.data() { return (opcodes, section.address()) }
      else { error!("Failed to get the opcodes for the ", s, " section.") }
    }
  }
  error!("Failed to find an executable section.")
}

#[inline(always)]
fn no_gadget_err(gadget : &[Const]) -> ! {
  let mut g = gadget.iter().map(|e|
    match e {
      Asm(asm, _, _) => asm.to_owned() + "\n",
    }
  ).collect::<String>();
  g.pop();
  error!("Failed to find a memory address for the gadget(s):\n", g);
}

fn mnemonicwise_search(gadget : &[Const], insns : &Instructions<'_>,
                       engine : &Keystone) -> String {
  for g in gadget {
    match g {
      Asm(asm, pos, _) => {
        let obj_code = assemble(engine, own!(asm), pos);
        for ins in insns.as_ref() {
          if obj_code.iter().eq(ins.bytes().iter()) {
            return string!(ins.address())
          }
        }
      },
    }
  }
  no_gadget_err(gadget)
}

fn bytewise_search(addr_offs : u64, gadget : &[Const], opcodes : &[u8],
                   engine : &Keystone) -> String {
  for g in gadget {
    match g {
      Asm(asm, pos, _) => {
        let obj_code = assemble(engine, own!(asm), pos);
        if let Some(gadget_offs) = kmp(opcodes, &obj_code) {
          return string!(addr_offs + gadget_offs as u64)
        }
      },
    }
  }
  no_gadget_err(gadget)
}

fn eval_gadget(opcodes : &[u8], gadget : &[Const], insns : &Instructions<'_>,
               engine : &Keystone, bytewise : bool,
               addr_offset : u64, outind : bool) -> String {
  let addr = if bytewise {
    bytewise_search(addr_offset, gadget, opcodes, engine)
  }
  else {
    mnemonicwise_search(gadget, insns, engine)
  };
  if outind { addr + "\n" }
  else      { addr        }
}

pub fn codegen(ast : &[AST], bin : String, archcpu : (Arch, Mode),
               syntax : (OptionValue, arch::x86::ArchSyntax),
               bytewise : bool, outind : bool) -> String {
  let data = read_bytes(&bin);

  let engine = Keystone::new(archcpu.0, archcpu.1)
                        .expect("Failed initializing the Keystone engine");

  if archcpu.0 == Arch::X86 {
    engine.option(OptionType::SYNTAX, syntax.0)
          .expect(&own!(format!("Could not set the x86 assembly syntax to: {:?}", syntax.0)));
  }

  let obj = match object::File::parse(&*data) {
    Ok(obj) => { obj }
    Err(e)  => { error!(format!("Failed to parse {} into object code: {}", bin, e)) }
  };

  let endianess = obj.endianness();
  if endianess == Endianness::Big {
    error!("big-endian architectures are not yet supported.");
  }

  let (opcodes, addr_offset) = get_opcodes_addr_offs(obj);

  let cs        = cs(archcpu, syntax.1);
  let mut insns = cs.disasm_all(&[], addr_offset)
                    .expect("Failed to disassemble");
  if !bytewise {
    insns = cs.disasm_all(opcodes, addr_offset)
              .expect("Failed to disassemble");
  }

  ast.iter().map(|n| {
    match n {
      Stat(Gadget(g)) => eval_gadget(opcodes, g, &insns, &engine, bytewise, addr_offset, outind),
      _ => string!("")
    }
  }).collect()
}
