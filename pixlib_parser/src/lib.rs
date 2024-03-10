pub mod common;
pub mod lexer;
pub mod parser;
pub mod scanner;

#[allow(clippy::assertions_on_constants)]
const _: () = assert!(usize::BITS >= u32::BITS);
