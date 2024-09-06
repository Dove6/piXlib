#![feature(get_mut_unchecked)]

pub mod common;
pub mod filesystems;
pub mod lexer;
pub mod parser;
pub mod runner;
pub mod scanner;

#[cfg(all(test, not(target_family = "wasm")))]
mod tests;

#[allow(clippy::assertions_on_constants)]
const _: () = assert!(usize::BITS >= u32::BITS);
