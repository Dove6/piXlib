use bevy::{
    asset::{Assets, Handle},
    log::{error, warn},
    math::Vec2,
    prelude::Image,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureFormat},
    },
    sprite::{Anchor, Sprite},
};
use std::ops::Add;

use pixlib_parser::{
    common::{Issue, IssueHandler, IssueKind},
    runner::{
        common::{ImageData, ImageDefinition, SpriteData},
        Rect,
    },
};

pub fn image_data_to_handle(
    textures: &mut Assets<Image>,
    image_definition: &ImageDefinition,
    image_data: &ImageData,
) -> Handle<Image> {
    textures.add(Image::new(
        Extent3d {
            width: image_definition.size_px.0,
            height: image_definition.size_px.1,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        image_data.data.to_vec(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    ))
}

pub fn animation_data_to_handle(
    textures: &mut Assets<Image>,
    rect: Rect,
    sprite_data: &SpriteData,
) -> Handle<Image> {
    textures.add(Image::new(
        Extent3d {
            width: rect.get_width() as u32,
            height: rect.get_height() as u32,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        sprite_data.data.to_vec(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    ))
}

#[derive(Debug)]
pub struct IssuePrinter;

impl<I: Issue> IssueHandler<I> for IssuePrinter {
    fn handle(&mut self, issue: I) {
        match issue.kind() {
            IssueKind::Warning => warn!("{:?}", issue),
            _ => error!("{:?}", issue),
        }
    }
}

pub fn add_tuples<TR, TL: Add<TR>>(
    a: (TL, TL),
    b: (TR, TR),
) -> (
    <TL as std::ops::Add<TR>>::Output,
    <TL as std::ops::Add<TR>>::Output,
) {
    (a.0 + b.0, a.1 + b.1)
}

pub fn get_anchor(offset: (i32, i32), size: (u32, u32)) -> (f32, f32) {
    (
        offset.0 as f32 / size.0 as f32,
        offset.1 as f32 / size.1 as f32,
    )
}

pub trait UpdatableAnchor {
    fn update_anchor(&mut self, offset_from_top_left: (f32, f32));
}

impl UpdatableAnchor for Sprite {
    fn update_anchor(&mut self, offset_from_top_left: (f32, f32)) {
        self.anchor = offset_by(Anchor::TopLeft, offset_from_top_left);
    }
}

fn offset_by(anchor: Anchor, offset: (f32, f32)) -> Anchor {
    Anchor::Custom(anchor.as_vec() + Vec2::new(-offset.0, offset.1))
}
