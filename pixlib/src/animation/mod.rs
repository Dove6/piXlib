mod animation_sequence_component;
mod animation_bundle;
mod animation_state;
mod animation_timer;
mod playback_state;

pub use animation_bundle::AnimationBundle;
pub use animation_bundle::AnimationMarker;
pub use animation_sequence_component::AnimationDefinition;
pub use animation_state::AnimationState;
pub use animation_timer::AnimationTimer;
pub use playback_state::PlaybackState;

use bevy::{
    asset::Assets,
    ecs::system::ResMut,
    math::{UVec2, Vec2},
    prelude::default,
    render::{render_resource::TextureFormat, texture::Image},
    sprite::{
        Sprite, SpriteSheetBundle, TextureAtlas, TextureAtlasBuilder, TextureAtlasBuilderError,
        TextureAtlasLayout,
    },
    time::{Timer, TimerMode},
};
use pixlib_formats::file_formats::ann::AnnFile;

use crate::{
    anchors::{add_tuples, get_anchor, UpdatableAnchor},
    image::image_data_to_image,
};

use self::animation_sequence_component::SpriteDefinition;

pub fn ann_file_to_animation_bundle(
    ann_file: &AnnFile,
    textures: &mut ResMut<Assets<Image>>,
    texture_atlases: &mut ResMut<Assets<TextureAtlasLayout>>,
) -> AnimationBundle {
    let (layout, texture) = build_texture_atlas(ann_file).unwrap();
    let layout = texture_atlases.add(layout);
    let texture = textures.add(texture);
    let mut sprite = Sprite::default();
    let animation = AnimationDefinition::new(ann_file);

    let first_non_empty_sequence_idx = animation
        .sequences
        .iter()
        .enumerate()
        .find(|(_, s)| !s.frames.is_empty())
        .map(|(i, _)| i);
    if let Some(first_non_empty_sequence_idx) = first_non_empty_sequence_idx {
        let frame = animation.sequences[first_non_empty_sequence_idx]
            .frames
            .first()
            .unwrap();
        let SpriteDefinition {
            offset_px, size_px, ..
        } = animation.sprites[frame.sprite_idx];
        sprite.update_anchor(get_anchor(add_tuples(offset_px, frame.offset_px), size_px));
    }

    let duration = 1.0f32 / ann_file.header.frames_per_second as f32;
    println!("Frame duration: {}", duration);
    let timer = AnimationTimer(Timer::from_seconds(duration, TimerMode::Repeating));

    AnimationBundle {
        marker: AnimationMarker,
        sprite_sheet: SpriteSheetBundle {
            texture,
            atlas: TextureAtlas { layout, index: 0 },
            sprite,
            ..default()
        },
        animation,
        state: AnimationState {
            sequence_idx: first_non_empty_sequence_idx.unwrap_or(0),
            ..default()
        },
        timer,
    }
}

fn build_texture_atlas(
    ann_file: &AnnFile,
) -> Result<(TextureAtlasLayout, Image), TextureAtlasBuilderError> {
    let mut texture_atlas_builder = TextureAtlasBuilder::default()
        .format(TextureFormat::Rgba8UnormSrgb)
        .auto_format_conversion(false)
        .padding(UVec2::new(1, 1))
        .max_size(Vec2::new(16384., 16384.));

    let textures: Vec<_> = ann_file
        .sprites
        .iter()
        .map(|sprite| {
            image_data_to_image(
                &sprite.image_data,
                (
                    sprite.header.width_px.into(),
                    sprite.header.height_px.into(),
                ),
                ann_file.header.color_format,
                sprite.header.compression_type,
                sprite.header.alpha_size_bytes > 0,
            )
        })
        .collect();

    for texture in textures.iter() {
        texture_atlas_builder.add_texture(None, texture);
    }
    texture_atlas_builder.finish()
}
