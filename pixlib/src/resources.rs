use bevy::ecs::system::Resource;

#[derive(Resource)]
pub struct WindowConfiguration {
    pub size: (usize, usize),
    pub title: &'static str,
}

#[derive(Resource, Default)]
pub struct DebugSettings {
    pub force_animation_infinite_looping: bool,
}
