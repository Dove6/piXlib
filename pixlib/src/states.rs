use bevy::ecs::schedule::States;

#[derive(States, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    SceneChooser,
    SceneViewer,
}

impl Default for AppState {
    fn default() -> Self {
        Self::SceneChooser
    }
}
