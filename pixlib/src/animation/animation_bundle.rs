use bevy::{
    ecs::{bundle::Bundle, component::Component},
    sprite::SpriteSheetBundle,
};

use super::{AnimationDefinition, AnimationState, AnimationTimer};

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnimationMarker;

#[derive(Bundle, Clone)]
pub struct AnimationBundle {
    pub marker: AnimationMarker,
    pub sprite_sheet: SpriteSheetBundle,
    pub animation: AnimationDefinition,
    pub state: AnimationState,
    pub timer: AnimationTimer,
}

impl std::fmt::Debug for AnimationBundle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnimationBundle")
            .field("marker", &self.marker)
            .field(
                "sprite_sheet",
                &(
                    &self.sprite_sheet.sprite,
                    &self.sprite_sheet.transform,
                    &self.sprite_sheet.global_transform,
                    &self.sprite_sheet.atlas,
                    &self.sprite_sheet.visibility,
                    &self.sprite_sheet.inherited_visibility,
                    &self.sprite_sheet.view_visibility,
                ),
            )
            .field("animation", &self.animation)
            .field("state", &self.state)
            .field("timer", &self.timer)
            .finish()
    }
}
