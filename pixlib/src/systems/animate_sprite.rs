use crate::anchors::{add_tuples, get_anchor, UpdatableAnchor};
use crate::animation::AnimationDefinition;
use crate::animation::CnvIdentifier;
use crate::resources::ScriptRunner;
use bevy::asset::{Assets, Handle};
use bevy::ecs::system::ResMut;
use bevy::log::{error, info, warn};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureFormat};
use bevy::render::texture::Image;
use bevy::render::view::Visibility;
use bevy::{ecs::system::Res, prelude::Query, sprite::Sprite, time::Time};
use pixlib_parser::classes::Animation;

pub fn animate_sprite(
    time: Res<Time>,
    mut script_runner: ResMut<ScriptRunner>,
    mut textures: ResMut<Assets<Image>>,
    mut query: Query<(
        &CnvIdentifier,
        &AnimationDefinition,
        &mut Sprite,
        &mut Visibility,
        &mut Handle<Image>,
    )>,
) {
    info!("Delta {:?}", time.delta());
    for (ident, _, mut atlas_sprite, mut visibility, mut texture) in &mut query {
        let Some(ident) = &ident.0 else {
            continue;
        };
        let Some(animation_obj_whole) = script_runner.read().unwrap().get_object(&ident) else {
            warn!(
                "Animation has no associated object in script runner: {:?}",
                ident
            );
            continue;
        };
        let animation_obj_whole_guard = animation_obj_whole.read().unwrap();
        let mut animation_obj_guard = animation_obj_whole_guard.content.write().unwrap();
        let animation_obj = animation_obj_guard
            .as_any_mut()
            .downcast_mut::<Animation>()
            .unwrap();
        animation_obj.tick(
            &mut pixlib_parser::runner::RunnerContext {
                runner: &mut *script_runner.write().unwrap(),
                self_object: ident.clone(),
                current_object: ident.clone(),
            },
            time.delta().as_secs_f64(),
        );
        let Ok(frame_to_show) = animation_obj
            .get_frame_to_show()
            .inspect_err(|e| error!("Error getting frame to show: {:?}", e))
        else {
            continue;
        };
        let Some((frame, sprite, data)) = frame_to_show else {
            *visibility = Visibility::Hidden;
            continue;
        };
        *texture = textures.add(Image::new(
            Extent3d {
                width: sprite.size_px.0,
                height: sprite.size_px.1,
                depth_or_array_layers: 1,
            },
            bevy::render::render_resource::TextureDimension::D2,
            data.data.to_owned(),
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        ));
        atlas_sprite.update_anchor(get_anchor(
            add_tuples(sprite.offset_px, frame.offset_px),
            sprite.size_px,
        ));
    }
}
