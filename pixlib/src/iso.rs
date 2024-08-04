use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};

use bevy::log::{error, info, warn};
use cdfs::{DirectoryEntry, ISO9660};
use pixlib_formats::file_formats::{
    ann::{parse_ann, AnnFile},
    arr::{parse_arr, ArrFile},
    img::{parse_img, ImgFile},
};
use pixlib_parser::common::{Issue, IssueHandler, IssueKind};

pub enum AmFile<'a> {
    Ann(AnnFile<'a>),
    Arr(ArrFile),
    Img(ImgFile<'a>),
    None,
}

pub fn read_file_from_iso(
    iso: &ISO9660<File>,
    filename: &Path,
    output_filename: Option<&str>,
) -> Vec<u8> {
    info!("PATH: {:?}", &filename);
    let mut buffer = Vec::<u8>::new();
    if let Ok(Some(DirectoryEntry::File(file))) =
        iso.open(&filename.as_os_str().to_str().unwrap().replace('\\', "/"))
    {
        let bytes_read = file.read().read_to_end(&mut buffer).unwrap();
        info!("Read file {:?} ({} bytes)", filename, bytes_read);
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
        "DTA" => {
            info!("Detected text database file.");
            AmFile::None
        }
        "FLD" => {
            info!("Detected numerical matrix file.");
            AmFile::None
        }
        "FNT" => {
            info!("Detected font file.");
            AmFile::None
        }
        "IMG" => AmFile::Img(parse_img(contents)),
        "INI" => {
            info!("Detected text configuration file.");
            AmFile::None
        }
        "LOG" => {
            info!("Detected log file.");
            AmFile::None
        }
        "SEQ" => {
            info!("Detected animation sequence description file.");
            AmFile::None
        }
        "WAV" => {
            info!("Detected audio file.");
            AmFile::None
        }
        _ => {
            info!("Unknown file type!");
            AmFile::None
        }
    }
}

#[derive(Debug)]
struct IssuePrinter;

impl<I: Issue> IssueHandler<I> for IssuePrinter {
    fn handle(&mut self, issue: I) {
        match issue.kind() {
            IssueKind::Warning => warn!("{:?}", issue),
            _ => error!("{:?}", issue),
        }
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
