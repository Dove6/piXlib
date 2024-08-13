use std::{
    cell::RefCell,
    env::Args,
    fs::File,
    io::Read,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    sync::Arc,
};

use bevy::{
    ecs::{entity::Entity, system::Resource},
    log::info,
};
use cdfs::{DirectoryEntry, ISOError, ISO9660};
use pixlib_parser::{
    classes::ObjectBuilderError,
    common::IssueManager,
    runner::{CnvRunner, FileSystem},
};

#[derive(Resource, Debug, Clone, PartialEq, Eq, Copy)]
pub struct WindowConfiguration {
    pub size: (usize, usize),
    pub title: &'static str,
}

#[derive(Resource, Default, Debug, Clone, PartialEq, Eq, Copy)]
pub struct DebugSettings {
    pub force_animation_infinite_looping: bool,
}

#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct ChosenEpisode(pub Option<Arc<Path>>);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SceneDefinition {
    pub name: String,
    pub path: PathBuf,
    pub background: Option<String>,
}

#[derive(Resource, Default, Debug, Clone)]
pub struct ChosenScene {
    pub list: Vec<SceneDefinition>,
    pub index: usize,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct RootEntityToDespawn(pub Option<Entity>);

#[derive(Resource, Debug, Default)]
pub struct ObjectBuilderIssueManager(pub IssueManager<ObjectBuilderError>);

impl Deref for ObjectBuilderIssueManager {
    type Target = IssueManager<ObjectBuilderError>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ObjectBuilderIssueManager {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone)]
pub struct ScriptRunner(pub Arc<CnvRunner>);

impl Deref for ScriptRunner {
    type Target = CnvRunner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<Arc<CnvRunner>> for ScriptRunner {
    fn as_ref(&self) -> &Arc<CnvRunner> {
        &self.0
    }
}

pub struct InsertedDiskResource(pub Arc<RefCell<InsertedDisk>>);

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
            return Err(std::io::Error::from(std::io::ErrorKind::NotFound));
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
