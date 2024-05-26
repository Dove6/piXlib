use std::{
    env::Args,
    fs::File,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    sync::Arc,
};

use bevy::ecs::{entity::Entity, system::Resource};
use cdfs::{ISOError, ISO9660};
use pixlib_parser::{classes::ObjectBuilderError, common::IssueManager, runner::CnvRunner};

#[derive(Resource, Debug, Clone, PartialEq, Eq, Copy)]
pub struct WindowConfiguration {
    pub size: (usize, usize),
    pub title: &'static str,
}

#[derive(Resource, Default, Debug, Clone, PartialEq, Eq, Copy)]
pub struct DebugSettings {
    pub force_animation_infinite_looping: bool,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct GamePaths {
    pub data_directory: PathBuf,
    pub game_definition_filename: PathBuf,
    pub music_directory: PathBuf,
    pub dialogues_directory: PathBuf,
    pub sfx_directory: PathBuf,
    pub common_directory: PathBuf,
    pub classes_directory: PathBuf,
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

#[derive(Resource, Debug, Default, Clone)]
pub struct ScriptRunner(pub CnvRunner);

impl Deref for ScriptRunner {
    type Target = CnvRunner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ScriptRunner {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Resource, Default)]
pub struct InsertedDisk {
    handle: Option<ISO9660<File>>,
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
