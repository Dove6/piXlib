use bevy::{
    ecs::component::Component,
    prelude::{Deref, DerefMut},
    time::Timer,
};

#[derive(Component, Deref, DerefMut, Clone, Debug, PartialEq, Eq, Default)]
pub struct AnimationTimer(pub Timer);
