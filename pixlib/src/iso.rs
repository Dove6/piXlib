use std::{
    fs::{self, File},
    io::Read,
    path::Path,
    sync::Arc,
};

use cdfs::{DirectoryEntry, ISO9660};
use pixlib_formats::file_formats::{
    ann::{parse_ann, AnnFile},
    arr::{parse_arr, ArrFile},
    img::{parse_img, ImgFile},
};
use pixlib_parser::{
    common::{Issue, IssueHandler, Position},
    runner::ScriptSource,
    scanner::{CnvDecoder, CnvHeader, CnvScanner, CodepageDecoder, CP1250_LUT},
};

use crate::resources::{GamePaths, ScriptRunner};

pub enum AmFile<'a> {
    Ann(AnnFile<'a>),
    Arr(ArrFile),
    Img(ImgFile<'a>),
    Cnv(CnvFile),
    None,
}

pub fn read_file_from_iso(
    iso: &ISO9660<File>,
    filename: &Path,
    output_filename: Option<&str>,
) -> Vec<u8> {
    println!("PATH: {:?}", &filename);
    let mut buffer = Vec::<u8>::new();
    if let Ok(Some(DirectoryEntry::File(file))) =
        iso.open(&filename.as_os_str().to_str().unwrap().replace('\\', "/"))
    {
        let bytes_read = file.read().read_to_end(&mut buffer).unwrap();
        println!("Read file {:?} ({} bytes)", filename, bytes_read);
    } else {
        panic!(
            "File not found: {}",
            &filename.as_os_str().to_str().unwrap()
        );
    }

    if let Some(output_path) = output_filename {
        fs::write(output_path, &buffer).expect("Could not write file");
    }

    buffer
}

pub fn parse_file<'a>(contents: &'a [u8], filename: &str) -> AmFile<'a> {
    let extension = filename
        .split('/')
        .last()
        .unwrap()
        .split('.')
        .last()
        .unwrap();

    match extension {
        "ANN" => AmFile::Ann(parse_ann(contents)),
        "ARR" => AmFile::Arr(parse_arr(contents)),
        "CLASS" | "CNV" | "DEF" => AmFile::Cnv(parse_cnv(contents)),
        "DTA" => {
            println!("Detected text database file.");
            AmFile::None
        }
        "FLD" => {
            println!("Detected numerical matrix file.");
            AmFile::None
        }
        "FNT" => {
            println!("Detected font file.");
            AmFile::None
        }
        "IMG" => AmFile::Img(parse_img(contents)),
        "INI" => {
            println!("Detected text configuration file.");
            AmFile::None
        }
        "LOG" => {
            println!("Detected log file.");
            AmFile::None
        }
        "SEQ" => {
            println!("Detected animation sequence description file.");
            AmFile::None
        }
        "WAV" => {
            println!("Detected audio file.");
            AmFile::None
        }
        _ => {
            println!("Unknown file type!");
            AmFile::None
        }
    }
}

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

#[derive(Debug, Clone)]
pub struct CnvFile(pub String);

pub fn parse_cnv(input: &[u8]) -> CnvFile {
    let mut input = input.iter().map(|b| Ok(*b)).peekable();
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
        })) => Box::new(CnvDecoder::new(
            input.collect::<Vec<_>>().into_iter(),
            step_count,
        )),
        Ok(None) => Box::new(
            first_line
                .into_iter()
                .map(Ok)
                .chain(input.collect::<Vec<_>>()),
        ),
        Err(err) => panic!("{}", err),
    };
    let decoder = CodepageDecoder::new(&CP1250_LUT, input);
    let scanner = CnvScanner::new(decoder);
    CnvFile(scanner.map(|r| r.unwrap().1).collect())
}

pub fn read_game_definition(
    iso: &ISO9660<File>,
    game_paths: &GamePaths,
    script_runner: &mut ScriptRunner,
) -> Arc<Path> {
    let mut buffer = Vec::<u8>::new();
    let mut game_definition_path = game_paths
        .data_directory
        .join(&game_paths.game_definition_filename);
    game_definition_path
        .as_mut_os_string()
        .make_ascii_uppercase();
    let game_definition_path: Arc<Path> = game_definition_path.into();
    if let Ok(Some(DirectoryEntry::File(file))) = iso.open(
        &game_definition_path
            .as_os_str()
            .to_str()
            .unwrap()
            .replace('\\', "/"),
    ) {
        let bytes_read = file.read().read_to_end(&mut buffer).unwrap();
        println!(
            "Read file {:?} ({} bytes)",
            game_definition_path, bytes_read
        );
    } else {
        panic!(
            "File not found: {}",
            &game_definition_path.as_os_str().to_str().unwrap()
        );
    }
    let result = parse_file(&buffer, game_definition_path.as_ref().to_str().unwrap());
    if let AmFile::Cnv(cnv_file) = result {
        if let Err(parsing_err) = script_runner.0.load_script(
            Arc::clone(&game_definition_path),
            cnv_file.0.char_indices().map(|(i, c)| {
                Ok((
                    Position {
                        line: 1,
                        column: 1 + i,
                        character: i,
                    },
                    c,
                    Position {
                        line: 1,
                        column: 2 + i,
                        character: i + 1,
                    },
                ))
            }),
            None,
            ScriptSource::Root,
        ) {
            panic!(
                "Error loading script {:?}: {}",
                &game_definition_path, parsing_err
            );
        }
        game_definition_path
    } else {
        panic!()
    }
}

pub fn build_data_path(
    path: &str,
    filename: &str,
    game_paths: &GamePaths,
    extension: Option<&str>,
) -> Arc<Path> {
    let mut script_path = game_paths
        .data_directory
        .join(path)
        .join(filename.to_owned() + extension.unwrap_or(""));
    script_path.as_mut_os_string().make_ascii_uppercase();
    script_path.into()
}

pub fn read_script(
    iso: &ISO9660<File>,
    path: &str,
    filename: &str,
    game_paths: &GamePaths,
    parent_path: Option<Arc<Path>>,
    script_source: ScriptSource,
    script_runner: &mut ScriptRunner,
) -> Arc<Path> {
    let mut buffer = Vec::<u8>::new();
    let script_path = build_data_path(path, filename, game_paths, Some(".CNV"));
    if let Ok(Some(DirectoryEntry::File(file))) =
        iso.open(&script_path.as_os_str().to_str().unwrap().replace('\\', "/"))
    {
        let bytes_read = file.read().read_to_end(&mut buffer).unwrap();
        println!("Read file {:?} ({} bytes)", script_path, bytes_read);
    } else {
        panic!(
            "File not found: {}",
            &script_path.as_os_str().to_str().unwrap()
        );
    }
    let result = parse_file(&buffer, script_path.as_ref().to_str().unwrap());
    if let AmFile::Cnv(cnv_file) = result {
        if let Err(parsing_err) = script_runner.0.load_script(
            Arc::clone(&script_path),
            cnv_file.0.char_indices().map(|(i, c)| {
                Ok((
                    Position {
                        line: 1,
                        column: 1 + i,
                        character: i,
                    },
                    c,
                    Position {
                        line: 1,
                        column: 2 + i,
                        character: i + 1,
                    },
                ))
            }),
            parent_path,
            script_source,
        ) {
            panic!("Error loading script {:?}: {}", &script_path, parsing_err);
        }
        script_path
    } else {
        panic!()
    }
}
