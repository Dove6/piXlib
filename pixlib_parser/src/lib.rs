use lalrpop_util::lalrpop_mod;

pub mod common;
pub mod lexer;
pub mod scanner;
lalrpop_mod!(pub parser);
pub mod ast;
#[allow(dead_code)]
pub mod classes;
pub mod declarative_parser;
pub mod runner;

#[allow(clippy::assertions_on_constants)]
const _: () = assert!(usize::BITS >= u32::BITS);
