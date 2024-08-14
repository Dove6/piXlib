use std::{
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    sync::Arc,
};

use bevy::ecs::{entity::Entity, system::Resource};
use pixlib_parser::{common::IssueManager, runner::ObjectBuilderError};

#[derive(Resource, Debug, Clone, PartialEq, Eq, Copy)]
pub struct WindowConfiguration {
    pub size: (usize, usize),
    pub title: &'static str,
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
