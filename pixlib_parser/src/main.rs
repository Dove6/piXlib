use std::{io::Read, path::PathBuf};

use pixlib_parser::{
    common::{Issue, IssueHandler, IssueManager},
    declarative_parser::{DeclarativeParser, ParserIssue},
    scanner::{CnvScanner, CodepageDecoder, CP1250_LUT},
};

#[derive(Debug)]
struct IssuePrinter;

impl<I: Issue> IssueHandler<I> for IssuePrinter {
    fn handle(&mut self, issue: I) {
        eprintln!("{:?}", issue);
    }
}

fn parse_declarative(filename: PathBuf) -> std::io::Result<()> {
    eprintln!("{:?}", &filename);
    let input = std::fs::File::open(filename)?;
    let decoder = CodepageDecoder::new(&CP1250_LUT, input.bytes());
    let scanner = CnvScanner::new(decoder);
    let mut parser_issue_manager: IssueManager<ParserIssue> = Default::default();
    parser_issue_manager.set_handler(Box::new(IssuePrinter));
    let mut dec_parser = DeclarativeParser::new(scanner, Default::default(), parser_issue_manager);
    println!("[STX]");
    while let Some(Ok((pos, dec, _))) = dec_parser.next() {
        println!("[{:?}] {:?}", pos, dec);
    }
    if let Some(Err(err)) = dec_parser.next() {
        println!("{:?}", err);
    }
    println!("[ETX]");
    Ok(())
}

fn main() -> std::io::Result<()> {
    for filename in std::env::args().skip(1) {
        parse_declarative(PathBuf::from(filename))?;
    }
    Ok(())
}

// let input = std::fs::File::open("code.txt")?;
// let decoder = CodepageDecoder::new(&CP1250_LUT, input.bytes());
// let scanner = CnvScanner::new(decoder);
// let mut lexer_issue_manager: IssueManager<LexerIssue> = Default::default();
// lexer_issue_manager.set_handler(Box::new(IssuePrinter));
// let mut lexer = CnvLexer::new(scanner, Default::default(), lexer_issue_manager);
// println!("[STX]");
// while let Some(Ok((pos, token, _))) = lexer.next() {
//     println!("[{:?}] {:?}", pos, token);
// }
// if let Some(Err(err)) = lexer.next() {
//     println!("{:?}", err);
// }
// println!("[ETX]");

// let input = std::fs::File::open("code.txt")?;
// let decoder = CodepageDecoder::new(&CP1250_LUT, input.bytes());
// let scanner = CnvScanner::new(decoder);
// let lexer = CnvLexer::new(scanner, Default::default(), Default::default());
// let mut parser_issue_manager: IssueManager<ParserIssue> = Default::default();
// parser_issue_manager.set_handler(Box::new(IssuePrinter));
// println!(
//     "{:#?}",
//     CodeParser::new().parse(&Default::default(), &mut parser_issue_manager, lexer)
// );
// Ok(())
