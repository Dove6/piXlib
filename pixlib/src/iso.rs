use std::{
    fs::{self, File},
    io::Read,
};

use opticaldisc::iso::IsoFs;
use pixlib_formats::file_formats::{
    ann::{parse_ann, AnnFile},
    arr::{parse_arr, ArrFile},
    img::{parse_img, ImgFile},
};

pub enum AmFile<'a> {
    Ann(AnnFile<'a>),
    Arr(ArrFile),
    Img(ImgFile<'a>),
    None,
}

pub fn read_iso(iso_file: &File) -> IsoFs<&File> {
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

pub fn read_file_from_iso(
    iso: &mut IsoFs<&File>,
    filename: &str,
    output_filename: Option<&str>,
) -> Vec<u8> {
    let mut buffer = Vec::<u8>::new();
    let bytes_read = iso
        .open_file(filename)
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();
    println!("Read file {} ({} bytes)", filename, bytes_read);

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
        "CLASS" | "CNV" | "DEF" => {
            println!("Detected script file.");
            AmFile::None
        }
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
