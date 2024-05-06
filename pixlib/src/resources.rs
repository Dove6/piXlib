use std::{env::Args, path::PathBuf};

use bevy::ecs::{entity::Entity, system::Resource};
use pixlib_parser::runner::CnvRunner;

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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SceneDefinition {
    pub name: String,
    pub path: PathBuf,
    pub background: Option<String>,
}

#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct ChosenScene {
    pub iso_file_path: Option<PathBuf>,
    pub scene_definition: Option<SceneDefinition>,
}

impl TryFrom<Args> for ChosenScene {
    type Error = ();

    fn try_from(args: Args) -> Result<Self, Self::Error> {
        let mut args = args.skip(1);
        let path_to_iso = args.next().ok_or(())?;
        Ok(Self {
            iso_file_path: Some(path_to_iso.into()),
            scene_definition: None,
        })
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct RootEntityToDespawn(pub Option<Entity>);

#[derive(Resource, Debug, Default, Clone)]
pub struct ScriptRunner(pub CnvRunner);
