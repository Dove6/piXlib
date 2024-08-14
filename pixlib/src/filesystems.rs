use std::{
    env::Args,
    fs::File,
    io::Read,
    path::Path,
    sync::{Arc, RwLock},
};

use bevy::{ecs::system::Resource, log::info};
use cdfs::{DirectoryEntry, ISOError, ISO9660};
use pixlib_parser::runner::FileSystem;
use zip::{result::ZipError, ZipArchive};

#[derive(Default, Debug)]
pub struct LayeredFileSystem {
    components: Vec<Arc<RwLock<dyn FileSystem>>>,
}

impl FileSystem for LayeredFileSystem {
    fn read_file(&mut self, filename: &str) -> std::io::Result<Vec<u8>> {
        for filesystem in self.components.iter().rev() {
            match filesystem.write().unwrap().read_file(filename) {
                Ok(v) => return Ok(v),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(e) => return Err(e),
            }
        }
        Err(std::io::Error::from(std::io::ErrorKind::NotFound))
    }

    fn write_file(&mut self, filename: &str, data: &[u8]) -> std::io::Result<()> {
        for filesystem in self.components.iter().rev() {
            match filesystem.write().unwrap().write_file(filename, data) {
                Err(e) if e.kind() == std::io::ErrorKind::Unsupported => continue,
                Err(e) => return Err(e),
                _ => return Ok(()),
            }
        }
        Err(std::io::Error::from(std::io::ErrorKind::Unsupported))
    }
}

impl LayeredFileSystem {
    pub fn new(base_fs: Arc<RwLock<dyn FileSystem>>) -> Self {
        Self {
            components: vec![base_fs],
        }
    }

    pub fn push_layer(&mut self, fs: Arc<RwLock<dyn FileSystem>>) {
        self.components.push(fs);
    }

    pub fn pop_layer(&mut self) -> Option<Arc<RwLock<dyn FileSystem>>> {
        if self.components.len() == 1 {
            return None;
        }
        self.components.pop()
    }
}

pub struct CompressedPatch {
    handle: ZipArchive<File>,
}

impl std::fmt::Debug for CompressedPatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompressedPatch")
            .field("handle", &"...")
            .finish()
    }
}

impl FileSystem for CompressedPatch {
    fn read_file(&mut self, filename: &str) -> std::io::Result<Vec<u8>> {
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
            .inspect_err(|e| eprintln!("{}", e))?;
        if entry.is_file() {
            let mut vec = Vec::new();
            entry.read_to_end(&mut vec)?;
            Ok(vec)
        } else {
            Err(std::io::Error::from(std::io::ErrorKind::NotFound))
        }
    }

    fn write_file(&mut self, _filename: &str, _data: &[u8]) -> std::io::Result<()> {
        Err(std::io::Error::from(std::io::ErrorKind::Unsupported))
    }
}

impl CompressedPatch {
    pub fn new(handle: File) -> Result<Self, ZipError> {
        Ok(Self {
            handle: ZipArchive::new(handle)?,
        })
    }
}

impl TryFrom<&Path> for CompressedPatch {
    type Error = ZipError;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let file = File::open(path).map_err(ZipError::Io)?;
        Self::new(file)
    }
}

#[derive(Resource)]
pub struct InsertedDiskResource(pub Arc<RwLock<InsertedDisk>>);

#[derive(Default)]
pub struct InsertedDisk {
    handle: Option<ISO9660<File>>,
}

impl std::fmt::Debug for InsertedDisk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InsertedDisk")
            .field("handle", &"...")
            .finish()
    }
}

impl FileSystem for InsertedDisk {
    fn read_file(&mut self, filename: &str) -> std::io::Result<Vec<u8>> {
        let Some(handle) = &self.handle else {
            return Err(std::io::Error::from(std::io::ErrorKind::Unsupported));
        };
        if let Ok(Some(DirectoryEntry::File(file))) =
            handle.open(&filename.replace('\\', "/").to_ascii_lowercase())
        {
            let mut buffer = Vec::new();
            let bytes_read = file.read().read_to_end(&mut buffer).unwrap();
            info!("Read file {:?} ({} bytes)", filename, bytes_read);
            Ok(buffer)
        } else {
            Err(std::io::Error::from(std::io::ErrorKind::NotFound))
        }
    }

    fn write_file(&mut self, _filename: &str, _data: &[u8]) -> std::io::Result<()> {
        Err(std::io::Error::from(std::io::ErrorKind::Unsupported))
    }
}

impl InsertedDisk {
    pub fn insert(&mut self, handle: File) -> Result<(), ISOError> {
        self.handle = Some(ISO9660::new(handle)?);
        Ok(())
    }

    pub fn eject(&mut self) {
        self.handle = None;
    }

    pub fn get(&self) -> Option<&ISO9660<File>> {
        self.handle.as_ref()
    }
}

impl TryFrom<Args> for InsertedDisk {
    type Error = ISOError;

    fn try_from(args: Args) -> Result<Self, Self::Error> {
        let mut args = args.skip(1);
        let path_to_iso = args.next().ok_or(ISOError::InvalidFs("Missing argument"))?;
        Ok(Self {
            handle: Some(ISO9660::new(File::open(path_to_iso)?)?),
        })
    }
}
