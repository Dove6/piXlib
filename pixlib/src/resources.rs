use std::{path::Path, sync::Arc};

use bevy::ecs::{entity::Entity, system::Resource};

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
}

#[derive(Resource, Default, Debug, Clone)]
pub struct ChosenScene {
    pub list: Vec<SceneDefinition>,
    pub index: usize,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct RootEntityToDespawn(pub Option<Entity>);
