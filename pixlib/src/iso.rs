use std::{
    collections::HashMap,
    fs::{self, File},
    io::Read,
    path::Path,
};

use opticaldisc::iso::IsoFs;
use pixlib_formats::file_formats::{
    ann::{parse_ann, AnnFile},
    arr::{parse_arr, ArrFile},
    img::{parse_img, ImgFile},
    DecodedStr,
};
use regex::Regex;

use crate::resources::GamePaths;

pub enum AmFile<'a> {
    Ann(AnnFile<'a>),
    Arr(ArrFile),
    Img(ImgFile<'a>),
    Cnv(CnvFile),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CnvFile(pub HashMap<String, HashMap<String, String>>);

fn parse_cnv(data: &[u8]) -> CnvFile {
    let object_definition_regex = Regex::new(r"(?i)^object\s*=\s*(.*)$").unwrap();
    let property_definition_regex =
        Regex::new(r"(?i)^([^:\s]*)\s*:\s*([^=\s]*)\s*=\s*(.*)$").unwrap();

    let mut objects = HashMap::new();
    for line in DecodedStr::from_bytes(&decode_cnv(data)).unwrap().0.lines() {
        let line = line.trim();
        // println!("Line: {line}");
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if let Some(object_captures) = object_definition_regex.captures(&line) {
            let object_name = object_captures.get(1).unwrap().as_str().to_uppercase();
            if objects.contains_key(&object_name) {
                panic!("Object {} re-declared!", &object_name);
            }
            objects.insert(object_name, HashMap::new());
        } else if let Some(property_captures) = property_definition_regex.captures(&line) {
            let object_name = property_captures.get(1).unwrap().as_str().to_uppercase();
            let property_name = property_captures.get(2).unwrap().as_str().to_uppercase();
            let property_value = property_captures.get(3).unwrap().as_str().to_owned();
            if !objects.contains_key(&object_name) {
                continue;  // TODO: don't ignore errors
            }
            let properties = objects.get_mut(&object_name).expect(&format!(
                "Trying to set property {} of undeclared object {}!",
                property_name, object_name
            ));
            if properties.contains_key(&property_name) {
                continue;  // TODO: don't ignore errors
                panic!(
                    "Property {} re-declared for object {}!",
                    &property_name, &object_name
                );
            }
            properties.insert(property_name, property_value);
        } else {
            panic!("Unexpected line: {}", line);
        }
    }
    CnvFile(objects)
}

pub fn read_game_definition(iso_file_path: &Path, game_paths: &GamePaths) -> CnvFile {
    let mut iso = opticaldisc::iso::IsoFs::new(File::open(&iso_file_path).unwrap()).unwrap();
    let mut buffer = Vec::<u8>::new();
    let game_definition_path = game_paths
        .data_directory
        .join(&game_paths.game_definition_filename);
    let _ = iso
        .open_file(&game_definition_path)
        .unwrap()
        .read_to_end(&mut buffer)
        .unwrap();
    let result = parse_file(&buffer, game_definition_path.to_str().unwrap());
    if let AmFile::Cnv(cnv_file) = result {
        cnv_file
    } else {
        panic!()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
struct CipherIterator<'a> {
    data: &'a [u8],
    index: usize,
    steps: u8,
    current_step: u8,
}

impl<'a> CipherIterator<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        let header_end = data
            .iter()
            .position(|v| matches!(*v, b'\n' | b'\r'))
            .unwrap_or(data.len());
        let header = &data[..header_end];
        if !(header.starts_with(b"{<C:") || header.starts_with(b"{<D:")) || !header.ends_with(b">}")
        {
            panic!();
        }
        let steps = &header[4..header.len() - 2];
        let steps = steps
            .iter()
            .filter(|digit| digit.is_ascii_digit() || panic!())
            .fold(0u32, |acc, digit| {
                acc * 10 + Into::<u32>::into(digit - b'0')
            }) as u8; // overflow acknowledged

        CipherIterator {
            data: &data[header_end..],
            index: 0,
            steps,
            current_step: 0,
        }
    }

    fn is_newline_token_ahead(&self) -> bool {
        self.data[self.index..].len() >= 3 && &self.data[self.index..self.index + 3] == b"<E>"
    }

    fn current_byte(&self) -> u8 {
        self.data[self.index]
    }

    fn current_byte_decoded(&self) -> u8 {
        let shift = (self.current_step >> 1) + 1;
        if self.current_step % 2 == 0 {
            self.current_byte() - shift
        } else {
            self.current_byte() + shift
        }
    }

    fn increment_step(&mut self) {
        self.current_step = (self.current_step + 1) % self.steps;
    }
}

impl<'a> Iterator for CipherIterator<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.data.len() && matches!(self.current_byte(), b'\r' | b'\n') {
            self.index += 1;
        }
        if self.index >= self.data.len() {
            return None;
        }
        if self.is_newline_token_ahead() {
            self.index += 3;
            return Some(b'\n');
        }
        let decoded_byte = self.current_byte_decoded();
        self.increment_step();
        self.index += 1;
        Some(decoded_byte)
    }
}

fn decode_cnv(data: &[u8]) -> Vec<u8> {
    CipherIterator::new(data).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cnv() {
        assert_eq!(decode_cnv(b"{<C:6>}\nPALCFQ"), b"OBJECT");
    }
}
