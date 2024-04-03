use lalrpop_util::lalrpop_mod;

pub mod common;
pub mod lexer;
lalrpop_mod!(pub parser);
pub mod scanner;

pub mod ast;

#[allow(clippy::assertions_on_constants)]
const _: () = assert!(usize::BITS >= u32::BITS);
