use std::{
    io::{Cursor, Read},
    path::PathBuf,
    sync::Arc,
};

use cdfs::{DirectoryEntry, ISOError, ISO9660};
use log::{error, info, trace};
use zip::{result::ZipError, ZipArchive};

use crate::runner::{FileSystem, Path};

#[derive(Debug)]
pub struct DummyFileSystem;

impl FileSystem for DummyFileSystem {
    fn read_file(&mut self, _: &str) -> std::io::Result<Arc<Vec<u8>>> {
        Ok(Arc::new(Vec::new()))
    }

    fn write_file(&mut self, _: &str, _: &[u8]) -> std::io::Result<()> {
        Ok(())
    }
}

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

#[cfg(not(target_family = "wasm"))]
impl GameDirectory {
    pub fn new(base_path: &str) -> std::io::Result<Self> {
        let res = GameDirectory {
            base_path: Path::from(base_path),
        };
        Self::get_matching_path(&res.base_path)?;
        Ok(res)
    }

    #[cfg(not(target_os = "windows"))]
    fn get_matching_path(path: &str) -> std::io::Result<PathBuf> {
        let path = Path::from(path);
        let mut built_path = String::from(if path.starts_with('/') { "/" } else { "." });
        let segment_count = path.split('/').count();
        for (i, segment) in path
            .split('/')
            .enumerate()
            .skip_while(|(_, s)| s.is_empty())
        {
            trace!("Matching path segment {segment}, currently built path is {built_path}");
            let Some(continuation) = std::fs::read_dir(&built_path)?
                .find(|r| {
                    r.as_ref().is_ok_and(|e| {
                        e.file_name().eq_ignore_ascii_case(segment)
                            && (i == segment_count - 1 || e.file_type().is_ok_and(|t| t.is_dir()))
                    })
                })
                .transpose()?
            else {
                return Err(std::io::Error::from(std::io::ErrorKind::NotFound));
            };
            if built_path == "." {
                built_path.clear();
            } else if built_path != "/" {
                built_path.push('/');
            }
            built_path.push_str(continuation.file_name().to_str().unwrap());
        }
        Ok(PathBuf::from(built_path))
    }

    #[cfg(target_os = "windows")]
    fn get_matching_path(path: &str) -> std::io::Result<PathBuf> {
        if std::fs::exists(path)? {
            Ok(PathBuf::from(path))
        } else {
            Err(std::io::Error::from(std::io::ErrorKind::NotFound))
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
