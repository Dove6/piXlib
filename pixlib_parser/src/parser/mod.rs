use lalrpop_util::lalrpop_mod;

pub mod ast;
pub mod declarative_parser;
lalrpop_mod!(pub imperative_parser, "/parser/imperative_parser.rs");
pub mod seq_parser;

#[cfg(test)]
mod imperative_parser_test {
    use std::vec::IntoIter;

    use crate::{
        common::{Issue, IssueHandler, IssueManager},
        lexer::CnvLexer,
        scanner::CnvScanner,
    };

    use super::*;
    use ast::{Expression, ParserIssue};
    use imperative_parser::*;

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
        let mut parser_issue_manager: IssueManager<ParserIssue> = Default::default();
        parser_issue_manager.set_handler(Box::new(IssuePrinter));
        let result = CodeParser::new()
            .parse(&Default::default(), &mut parser_issue_manager, lexer)
            .unwrap();
        println!("{:?}", result);
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

    #[derive(Debug)]
    struct IssuePrinter;

    impl<I: Issue> IssueHandler<I> for IssuePrinter {
        fn handle(&mut self, issue: I) {
            eprintln!("{:?}", issue);
        }
    }
}
