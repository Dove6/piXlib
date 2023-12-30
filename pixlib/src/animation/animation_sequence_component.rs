use bevy::ecs::component::Component;
use pixlib_formats::file_formats::ann::{self, AnnFile, LoopingSettings};

#[derive(Component, Clone, Debug, PartialEq, Eq)]
pub struct AnimationDefinition {
    pub sequences: Vec<SequenceDefinition>,
    pub sprites: Vec<SpriteDefinition>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SequenceDefinition {
    pub name: String,
    pub opacity: u8,
    pub looping: LoopingSettings,
    pub frames: Vec<FrameDefinition>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameDefinition {
    pub name: String,
    pub offset_px: (i32, i32),
    pub opacity: u8,
    pub sprite_idx: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpriteDefinition {
    pub name: String,
    pub size_px: (u32, u32),
    pub offset_px: (i32, i32),
}

impl AnimationDefinition {
    pub fn new(ann_file: &AnnFile) -> Self {
        let sequences = ann_file
            .sequences
            .iter()
            .map(|sequence: &ann::Sequence| SequenceDefinition {
                name: sequence.header.name.0.clone(),
                opacity: sequence.header.opacity,
                looping: sequence.header.looping,
                frames: sequence
                    .frames
                    .iter()
                    .zip(&sequence.header.frame_to_sprite_mapping)
                    .map(|(frame, sprite_idx)| FrameDefinition {
                        name: frame.name.0.clone(),
                        offset_px: (frame.x_position_px.into(), frame.y_position_px.into()),
                        opacity: frame.opacity,
                        sprite_idx: (*sprite_idx).into(),
                    })
                    .collect(),
            })
            .collect();
        let sprites = ann_file
            .sprites
            .iter()
            .map(|sprite| SpriteDefinition {
                name: sprite.header.name.0.clone(),
                size_px: (
                    sprite.header.width_px.into(),
                    sprite.header.height_px.into(),
                ),
                offset_px: (
                    sprite.header.x_position_px.into(),
                    sprite.header.y_position_px.into(),
                ),
            })
            .collect();
        Self { sequences, sprites }
    }
}
