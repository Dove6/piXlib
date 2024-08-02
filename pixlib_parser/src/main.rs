use std::{
    cell::RefCell,
    collections::HashMap,
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};

use pixlib_parser::{
    classes::{CnvObject, CnvObjectBuilder},
    common::{Issue, IssueHandler, IssueManager},
    declarative_parser::{CnvDeclaration, DeclarativeParser, ParserIssue},
    runner::{CnvRunner, CnvScript, DummyFileSystem, FileSystem},
    scanner::{CnvDecoder, CnvHeader, CnvScanner, CodepageDecoder, CP1250_LUT},
};

#[derive(Debug)]
struct IssuePrinter;

impl<I: Issue> IssueHandler<I> for IssuePrinter {
    fn handle(&mut self, issue: I) {
        eprintln!("{:?}", issue);
    }
}

trait SomePanicable {
    fn and_panic(&self);
}

impl<T> SomePanicable for Option<T> {
    fn and_panic(&self) {
        if self.is_some() {
            panic!();
        }
    }
}

#[derive(Debug)]
struct PlainFileSystem;

impl FileSystem for PlainFileSystem {
    fn read_file(&self, filename: &str) -> std::io::Result<Vec<u8>> {
        std::fs::read(filename)
    }

    fn write_file(&mut self, filename: &str, data: &[u8]) -> std::io::Result<()> {
        std::fs::write(filename, data)
    }
}

fn parse_declarative(filename: PathBuf) -> std::io::Result<()> {
    eprintln!("{:?}", &filename);
    let input = std::fs::File::open(filename.clone())?;
    let mut input = input.bytes().peekable();
    let mut first_line = Vec::<u8>::new();
    while let Some(res) =
        input.next_if(|res| res.as_ref().is_ok_and(|c| !matches!(c, b'\r' | b'\n')))
    {
        first_line.push(res.unwrap())
    }
    while let Some(res) =
        input.next_if(|res| res.as_ref().is_ok_and(|c| matches!(c, b'\r' | b'\n')))
    {
        first_line.push(res.unwrap())
    }
    let input: Box<dyn Iterator<Item = std::io::Result<u8>>> = match CnvHeader::try_new(&first_line)
    {
        Ok(Some(CnvHeader {
            cipher_class: _,
            step_count,
        })) => Box::new(CnvDecoder::new(input, step_count)),
        Ok(None) => Box::new(first_line.into_iter().map(Ok).chain(input)),
        Err(err) => panic!("{}", err),
    };
    let decoder = CodepageDecoder::new(&CP1250_LUT, input);
    let scanner = CnvScanner::new(decoder);
    let mut parser_issue_manager: IssueManager<ParserIssue> = Default::default();
    parser_issue_manager.set_handler(Box::new(IssuePrinter));
    let mut dec_parser =
        DeclarativeParser::new(scanner, Default::default(), parser_issue_manager).peekable();
    let mut objects: HashMap<String, CnvObjectBuilder> = HashMap::new();
    println!("Starting parsing...");
    let mut counter: usize = 0;
    while let Some(Ok((_pos, dec, _))) = dec_parser.next_if(|result| result.is_ok()) {
        match dec {
            CnvDeclaration::ObjectInitialization(name) => {
                objects
                    .insert(
                        name.clone(),
                        CnvObjectBuilder::new(
                            Arc::new(RefCell::new(CnvScript {
                                source_kind: pixlib_parser::runner::ScriptSource::Scene,
                                path: Path::new("").into(),
                                parent_path: None,
                                objects: Vec::new(),
                                runner: Arc::new(RefCell::new(CnvRunner {
                                    scripts: HashMap::new(),
                                    filesystem: Arc::new(RefCell::new(DummyFileSystem {})),
                                    current_scene: None,
                                })),
                            })),
                            filename.clone().into(),
                            name,
                            counter,
                        ),
                    )
                    .and_panic();
                counter += 1;
            }
            CnvDeclaration::PropertyAssignment {
                parent,
                property,
                property_key: _property_key,
                value,
            } => {
                let Some(obj) = objects.get_mut(&parent) else {
                    panic!(
                        "Expected {} element to be in dict, the element list is: {:?}",
                        &parent, &objects
                    );
                };
                obj.add_property(property, value);
            }
        }
    }
    if let Some(Err(err)) = dec_parser.next_if(|result| result.is_err()) {
        println!("{:?}", err);
    }
    println!("Parsing ended. Building objects.");
    let filesystem = PlainFileSystem;
    let objects: HashMap<String, Arc<CnvObject>> = objects
        .into_iter()
        .map(|(name, builder)| (name, builder.build().unwrap()))
        .collect();
    println!("Built objects:");
    for obj in objects {
        println!("{:#?}", obj.1.name);
    }
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
