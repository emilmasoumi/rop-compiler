/*

The symbol table.

*/

use ast::*;

use self::SymTab::*;

#[derive(Debug, Clone)]
pub enum SymTab {
    Entry(Id, Type)
}

pub fn lookup(symtab : Vec<SymTab>, id_in : Id) -> Option<SymTab> {
  for e in symtab {
    match e {
      Entry(id, ty) => {
        if id == id_in {
          return Some(Entry(id, ty));
        }
      }
      _ => {()}
    }
  }
  return None;
}

pub fn pp_symtab(symtab : Vec<SymTab>) -> () {
  println!("-------- Printing the symbol table --------");
  for entry in symtab {
    println!("{:?}", entry);
  }
}
