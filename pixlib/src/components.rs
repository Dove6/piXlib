use bevy::ecs::component::Component;

#[derive(Component, Clone, Debug, PartialEq, Eq)]
pub struct ObjectIdentifier(pub String);
