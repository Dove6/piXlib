use lalrpop_util::lalrpop_mod;

pub mod ast;
pub mod declarative_parser;
lalrpop_mod!(pub imperative_parser, "/parser/imperative_parser.rs");
