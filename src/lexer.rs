/*

Lexical analysis of the source code where its sequence of characters is turned into a
sequence of tokens depicted by an algebraic type `Token`.

*/

use utils::{err_line};

use self::Token::*;
use self::Pos::*;

#[derive(Debug, Clone)]
pub enum Pos {
  Pos(usize, usize),
}

#[derive(Debug, Clone)]
pub enum Token {
  BlockStart   (Pos),
  BlockEnd     (Pos),
  Comma        (Pos),
  CommentStart (Pos),
  CommentEnd   (Pos),
  CommentLine  (Pos),
  EqualTok     (Pos),
  Identifier   (Pos, char),
  StringStart  (Pos),
  StringEnd    (Pos),
  Semicolon    (Pos),
  AssemblyTok  (Pos, char),
  Whitespace   (Pos),
}

/*
  Lexical analysis of the entire source code stored in `src`.
*/
pub fn lexer(src: &String) -> Vec<Token> {
  if src.len() < 1 {
    error!("cannot process empty file: ", src);
  }
  let mut parse_tree : Vec<Token> = Vec::new();

  let mut is_comment        = false;
  let mut _is_multi_comment = false;

  let mut line_n = 1;
  let mut col_n  = 0;
  let mut is_asm = false;
  let mut it     = src.chars().peekable();

  while let Some(&c) = it.peek() {
    if c == '\n' {
      line_n     = line_n + 1;
      col_n      = 0;
      is_comment = false;
    }
    col_n = col_n + 1;
    if is_asm {
      match c {
        '\\' => {
          parse_tree.push(AssemblyTok(Pos(line_n, col_n), c));
          it.next();
          if let Some(&a) = it.peek() {
            parse_tree.push(AssemblyTok(Pos(line_n, col_n), a));
          } else {
            error!("lexer(): unexpected error when processing: \'\\\':\n",
                   err_line(src, &"\\".to_string(), &Pos(line_n, col_n)));
          }
        }
        '\"' => {
          parse_tree.push(StringEnd(Pos(line_n, col_n)));
          is_asm = false;
        }
        _ => {
          parse_tree.push(AssemblyTok(Pos(line_n, col_n), c));
        }
      }
      it.next();
      continue;
    }
    if is_comment {
      continue;
    }
    match c {
      '0'..='9' | 'A'..='Z' | 'a'..='z' => {
        parse_tree.push(Identifier(Pos(line_n, col_n), c));
      }
      '{' => {
        parse_tree.push(BlockStart(Pos(line_n, col_n)));
      }
      '}' => {
        parse_tree.push(BlockEnd(Pos(line_n, col_n)));
      }
      ',' => {
        parse_tree.push(Comma(Pos(line_n, col_n)));
      }
      ';' => {
        parse_tree.push(Semicolon(Pos(line_n, col_n)));
      }
      '=' => {
        parse_tree.push(EqualTok(Pos(line_n, col_n)));
      }
      '\"' => {
        parse_tree.push(StringStart(Pos(line_n, col_n)));
        is_asm = true;
      }
      '\n' => {
        parse_tree.push(Whitespace(Pos(line_n, col_n)));
      }
      c if c.is_whitespace() => {
        parse_tree.push(Whitespace(Pos(line_n, col_n)));
      }
      _ => {
        error!("unexpected character: ", c);
      }
    }
    it.next();
  }
  return parse_tree;
}
