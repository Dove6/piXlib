use bevy::{
    ecs::system::{Commands, ResMut},
    hierarchy::DespawnRecursiveExt,
};

use crate::resources::RootEntityToDespawn;

pub fn cleanup_root(mut commands: Commands, mut root_entity: ResMut<RootEntityToDespawn>) {
    if let Some(entity) = root_entity.0.take() {
        commands.entity(entity).despawn_recursive();
    }
}
