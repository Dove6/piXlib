use std::{
    io::{Cursor, Read},
    path::PathBuf,
    sync::Arc,
};

use cdfs::{DirectoryEntry, ISOError, ISO9660};
#[cfg(not(target_family = "wasm"))]
use glob::{glob_with, GlobError, MatchOptions, Pattern, PatternError};
use log::{error, info, trace};
use thiserror::Error;
use zip::{result::ZipError, ZipArchive};

use crate::runner::{FileSystem, Path};

pub struct CompressedPatch {
    handle: ZipArchive<Cursor<Vec<u8>>>,
}

impl std::fmt::Debug for CompressedPatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompressedPatch")
            .field("handle", &"...")
            .finish()
    }
}

impl FileSystem for CompressedPatch {
    fn read_file(&mut self, filename: &str) -> std::io::Result<Arc<Vec<u8>>> {
        let sought_name = self
            .handle
            .file_names()
            .find(|n| n.eq_ignore_ascii_case(filename))
            .map(|s| s.to_owned());
        let Some(sought_name) = sought_name else {
            return Err(std::io::Error::from(std::io::ErrorKind::NotFound));
        };
        let mut entry = self
            .handle
            .by_name(&sought_name)
            .map_err(|e| match e {
                ZipError::FileNotFound => std::io::Error::from(std::io::ErrorKind::NotFound),
                ZipError::Io(io_error) => io_error,
                _ => std::io::Error::from(std::io::ErrorKind::Other),
            })
            .inspect_err(|e| error!("{}", e))?;
        if entry.is_file() {
            let mut wrapped_vec = Arc::new(Vec::new());
            let vec = Arc::get_mut(&mut wrapped_vec).unwrap();
            entry.read_to_end(vec)?;
            Ok(wrapped_vec)
        } else {
            Err(std::io::Error::from(std::io::ErrorKind::NotFound))
        }
    }

    fn write_file(&mut self, _filename: &str, _data: &[u8]) -> std::io::Result<()> {
        Err(std::io::Error::from(std::io::ErrorKind::Unsupported))
    }
}

impl CompressedPatch {
    pub fn new(data: Vec<u8>) -> Result<Self, ZipError> {
        Ok(Self {
            handle: ZipArchive::new(Cursor::new(data))?,
        })
    }
}

pub struct InsertedDisk {
    handle: ISO9660<Cursor<Vec<u8>>>,
}

impl std::fmt::Debug for InsertedDisk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InsertedDisk")
            .field("handle", &"...")
            .finish()
    }
}

impl FileSystem for InsertedDisk {
    fn read_file(&mut self, filename: &str) -> std::io::Result<Arc<Vec<u8>>> {
        let handle = &self.handle;
        if let Ok(Some(DirectoryEntry::File(file))) =
            handle.open(&filename.replace('\\', "/").to_ascii_lowercase())
        {
            let mut wrapped_buffer = Arc::new(Vec::new());
            let buffer = Arc::get_mut(&mut wrapped_buffer).unwrap();
            let bytes_read = file.read().read_to_end(buffer).unwrap();
            info!("Read file {:?} ({} bytes)", filename, bytes_read);
            Ok(wrapped_buffer)
        } else {
            Err(std::io::Error::from(std::io::ErrorKind::NotFound))
        }
    }

    fn write_file(&mut self, _filename: &str, _data: &[u8]) -> std::io::Result<()> {
        Err(std::io::Error::from(std::io::ErrorKind::Unsupported))
    }
}

impl InsertedDisk {
    pub fn new(data: Vec<u8>) -> Result<Self, ISOError> {
        Ok(Self {
            handle: ISO9660::new(Cursor::new(data))?,
        })
    }
}

#[cfg(not(target_family = "wasm"))]
#[derive(Debug)]
pub struct GameDirectory {
    base_path: Path,
}

#[derive(Debug, Error)]
pub enum GameDirectoryError {
    #[cfg(not(target_family = "wasm"))]
    #[error("Incorrect glob pattern: {0}")]
    GlobPattern(PatternError),
    #[cfg(not(target_family = "wasm"))]
    #[error("Error while iterating over glob results: {0}")]
    GlobIteration(GlobError),
    #[error("I/O error: {0}")]
    Io(std::io::Error),
}

#[cfg(not(target_family = "wasm"))]
impl From<PatternError> for GameDirectoryError {
    fn from(value: PatternError) -> Self {
        Self::GlobPattern(value)
    }
}

#[cfg(not(target_family = "wasm"))]
impl From<GlobError> for GameDirectoryError {
    fn from(value: GlobError) -> Self {
        Self::GlobIteration(value)
    }
}

impl From<std::io::Error> for GameDirectoryError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[cfg(not(target_family = "wasm"))]
impl GameDirectory {
    pub fn new(base_path: &str) -> Result<Self, GameDirectoryError> {
        let res = GameDirectory {
            base_path: Path::from(base_path),
        };
        Self::get_matching_path(&res.base_path)?;
        Ok(res)
    }

    fn get_matching_path(path: &str) -> std::io::Result<PathBuf> {
        let mut iter = glob_with(
            &Pattern::escape(path),
            MatchOptions {
                case_sensitive: false,
                ..MatchOptions::default()
            },
        )
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?
        .inspect(|r| {
            if let Err(e) = r {
                error!("Error glob-matching path: {e}")
            }
        })
        .filter_map(|r| r.ok());
        let result = iter
            .next()
            .ok_or(std::io::Error::from(std::io::ErrorKind::NotFound));
        if iter.next().is_some() {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Ambiguous glob",
            ))
        } else {
            result
        }
    }
}

#[cfg(not(target_family = "wasm"))]
impl FileSystem for GameDirectory {
    fn read_file(&mut self, filename: &str) -> std::io::Result<Arc<Vec<u8>>> {
        let matched_path = Self::get_matching_path(&self.base_path.with_appended(filename))?;
        let mut file = std::fs::File::open(matched_path)?;
        let mut wrapped_vec = Arc::new(Vec::new());
        let vec = Arc::get_mut(&mut wrapped_vec).unwrap();
        file.read_to_end(vec)?;
        Ok(wrapped_vec)
    }

    fn write_file(&mut self, filename: &str, data: &[u8]) -> std::io::Result<()> {
        trace!("Writing to {} data: {:?}", filename, data);
        let total_path = self.base_path.with_appended(filename);
        if let Ok(writing_path) = Self::get_matching_path(&total_path) {
            trace!("Matched path: {:?}", writing_path);
            return std::fs::write(writing_path, data);
        }
        let (rest_index, mut max_matching_path) = total_path
            .rmatch_indices('/')
            .filter_map(
                |(i, _)| match Self::get_matching_path(&total_path[..(i + 1)]) {
                    Ok(path) => Some((i + 1, path)),
                    Err(_) => None,
                },
            )
            .next()
            .unwrap_or((0, PathBuf::from("./")));
        trace!("Max matching path: {:?}", max_matching_path);
        max_matching_path.push(&total_path[rest_index..]);
        if let Some(parent) = max_matching_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(max_matching_path, data)
    }
}
