use crate::resources::WindowConfiguration;
use bevy::{
    ecs::system::Res, math::Vec3, prelude::{default, Camera2dBundle, Commands, Transform}
};

pub fn setup(window_config: Res<WindowConfiguration>, mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(
            window_config.size.0 as f32 / 2.0,
            window_config.size.1 as f32 / 2.0,
            0.0,
        ).with_scale(Vec3::new(1f32, -1f32, 1f32)),
        ..default()
    });
}
