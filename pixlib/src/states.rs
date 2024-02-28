use bevy::ecs::schedule::States;

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    SceneChooser,
    SceneViewer,
}
