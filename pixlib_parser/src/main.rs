use std::io::Read;

use pixlib_parser::{
    common::Token,
    lexer::CnvLexer,
    scanner::{CnvScanner, CodepageDecoder, CP1250_LUT},
};

fn main() -> std::io::Result<()> {
    let input = std::fs::File::open("code.txt")?;
    let decoder = CodepageDecoder::new(&CP1250_LUT, input.bytes());
    let scanner = CnvScanner::new(decoder);
    let mut lexer = CnvLexer::new(scanner, Default::default());
    println!("[STX]");
    while let Some(Ok(Token {
        value,
        bounds,
        had_errors,
    })) = lexer.next()
    {
        println!("{:?}", value);
        println!("    {:?}", bounds);
        if had_errors {
            println!("    (Had errors.)");
        }
    }
    if let Some(Err(err)) = lexer.next() {
        println!("{:?}", err);
    }
    println!("[ETX]");
    Ok(())
}
