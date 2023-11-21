mod arr_parser;

use crate::arr_parser::describe_arr;
use std::{fs, io::Read};

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 3 {
        panic!("Usage: iso_browser path_to_iso path_to_file_on_iso [output_path]");
    }
    let path_to_iso = args[1].clone();
    let path_to_file = args[2].to_ascii_uppercase();
    let output_path = args.get(3);

    let file: std::fs::File = std::fs::File::open(&path_to_iso).unwrap();
    let mut iso = opticaldisc::iso::IsoFs::new(file).unwrap();

    for entry in iso.read_dir("/").unwrap().iter() {
        println!(
            "Entry discovered: {}, is file? {}",
            &entry.name(),
            entry.is_file()
        );
    }

    let mut buffer = Vec::<u8>::new();
    let bytes_read = iso
        .open_file(&path_to_file)
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();
    println!(
        "Read file {} ({} bytes) from disk {}",
        &path_to_file, bytes_read, &path_to_iso
    );

    if let Some(output_path) = output_path {
        fs::write(&output_path, &buffer).expect("Could not write file");
    }

    let extension = path_to_file
        .split('/')
        .last()
        .unwrap()
        .split('.')
        .last()
        .unwrap();

    match extension {
        "ANN" => println!("Detected animation file."),
        "ARR" => {
            describe_arr(&buffer);
        }
        "CLASS" | "CNV" | "DEF" => println!("Detected script file."),
        "DTA" => describe_dta(&buffer),
        "FLD" => println!("Detected numerical matrix file."),
        "FNT" => println!("Detected font file."),
        "IMG" => describe_img(&buffer),
        "INI" => println!("Detected text configuration file."),
        "LOG" => println!("Detected log file."),
        "SEQ" => describe_seq(&buffer),
        "WAV" => println!("Detected audio file."),
        _ => println!("Unknown file type!"),
    }
}

fn describe_dta(data: &Vec<u8>) {
    println!("Detected text database file.");
}

fn describe_img(data: &Vec<u8>) {
    println!("Detected static image file.");
}

fn describe_seq(data: &Vec<u8>) {
    println!("Detected animation sequence description file.");
}
