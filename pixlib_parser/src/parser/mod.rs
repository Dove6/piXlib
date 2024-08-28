use lalrpop_util::lalrpop_mod;

pub mod ast;
pub mod declarative_parser;
lalrpop_mod!(pub imperative_parser, "/parser/imperative_parser.rs");
pub mod seq_parser;

#[cfg(test)]
mod imperative_parser_test {
    use std::vec::IntoIter;

    use crate::{lexer::CnvLexer, scanner::CnvScanner};

    use super::*;
    use ast::Expression;
    use imperative_parser::*;
    use log::info;

    #[test]
    fn test_syntex_sugar_for_parametrized_event_handler() {
        let code_to_parse = "BEH_FOLLOW(REKSIO17A)";
        let scanner = CnvScanner::<IntoIter<_>>::new(
            code_to_parse
                .chars()
                .map(Ok)
                .collect::<Vec<_>>()
                .into_iter(),
        );
        let lexer = CnvLexer::new(scanner, Default::default(), Default::default());
        let result = CodeParser::new().parse(&Default::default(), lexer).unwrap();
        info!("{:?}", result);
        let Expression::Invocation(invocation) = result.value else {
            panic!();
        };
        assert_eq!(
            invocation.parent,
            Some(Expression::Identifier("BEH_FOLLOW".into()))
        );
        assert_eq!(invocation.name, "RUNC");
        assert_eq!(
            invocation.arguments,
            vec![Expression::Identifier("REKSIO17A".into())]
        );
    }
}
