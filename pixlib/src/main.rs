mod arr_parser;
mod img_parser;

use arr_parser::ArrFile;
use opticaldisc::iso::IsoFs;

use crate::arr_parser::parse_arr;
use crate::img_parser::parse_img;
use std::{
    fs::{self, File},
    io::Read,
};

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 3 {
        panic!("Usage: iso_browser path_to_iso path_to_file_on_iso [output_path]");
    }
    let path_to_iso = args[1].clone();
    let path_to_file = args[2].to_ascii_uppercase();
    let output_path = args.get(3);

    let iso_file = File::open(&path_to_iso).unwrap();
    let mut iso = read_iso(&iso_file);
    let parsed_file = parse_file_from_iso(&mut iso, &path_to_file, output_path.map(|v| v.as_ref()));
}

fn read_iso(iso_file: &File) -> IsoFs<&File> {
    let mut iso = opticaldisc::iso::IsoFs::new(iso_file).unwrap();

    println!("Loaded ISO file.");
    for entry in iso.read_dir("/").unwrap().iter() {
        println!(
            "Entry discovered: {}, is file? {}",
            &entry.name(),
            entry.is_file()
        );
    }

    iso
}

fn parse_file_from_iso(iso: &mut IsoFs<&File>, filename: &str, output_filename: Option<&str>) -> AmFile {
    let mut buffer = Vec::<u8>::new();
    let bytes_read = iso
        .open_file(&filename)
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();
    println!("Read file {} ({} bytes)", filename, bytes_read);

    if let Some(output_path) = output_filename {
        fs::write(&output_path, &buffer).expect("Could not write file");
    }

    let extension = filename
        .split('/')
        .last()
        .unwrap()
        .split('.')
        .last()
        .unwrap();

    match extension {
        "ANN" => {
            println!("Detected animation file.");
            AmFile::None
        },
        "ARR" => AmFile::Arr(parse_arr(&buffer)),
        "CLASS" | "CNV" | "DEF" => {
            println!("Detected script file.");
            AmFile::None
        },
        "DTA" => {
            parse_dta(&buffer);
            AmFile::None
        },
        "FLD" => {
            println!("Detected numerical matrix file.");
            AmFile::None
        },
        "FNT" => {
            println!("Detected font file.");
            AmFile::None
        },
        "IMG" => {
            parse_img(&buffer);
            AmFile::None
        },
        "INI" => {
            println!("Detected text configuration file.");
            AmFile::None
        },
        "LOG" => {
            println!("Detected log file.");
            AmFile::None
        },
        "SEQ" => {
            parse_seq(&buffer);
            AmFile::None
        },
        "WAV" => {
            println!("Detected audio file.");
            AmFile::None
        },
        _ => {
            println!("Unknown file type!");
            AmFile::None
        },
    }
}

fn parse_dta(data: &Vec<u8>) {
    println!("Detected text database file.");
}

fn parse_seq(data: &Vec<u8>) {
    println!("Detected animation sequence description file.");
}

enum AmFile {
    Arr(ArrFile),
    None,
}
