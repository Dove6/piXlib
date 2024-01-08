use bevy::{
    ecs::system::{Commands, Res},
    hierarchy::DespawnRecursiveExt,
};

use crate::resources::RootEntityToDespawn;

pub fn cleanup_root(mut commands: Commands, root_entity: Res<RootEntityToDespawn>) {
    commands.entity(root_entity.0).despawn_recursive();
}
