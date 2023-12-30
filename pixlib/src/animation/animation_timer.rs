use bevy::{time::Timer, prelude::{DerefMut, Deref}, ecs::component::Component};

#[derive(Component, Deref, DerefMut, Clone, Debug, PartialEq, Eq)]
pub struct AnimationTimer(pub Timer);
