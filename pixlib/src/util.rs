use bevy::{
    asset::{Assets, Handle},
    prelude::Image,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureFormat},
    },
};
use pixlib_formats::file_formats::img::ImgFile;
use pixlib_parser::classes::{ImageData, ImageDefinition, SpriteData, SpriteDefinition};

pub fn img_file_to_handle(textures: &mut Assets<Image>, file: ImgFile) -> Handle<Image> {
    textures.add(Image::new(
        Extent3d {
            width: file.header.width_px,
            height: file.header.height_px,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        file.image_data
            .to_rgba8888(file.header.color_format, file.header.compression_type)
            .to_vec(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    ))
}

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
    sprite_definition: &SpriteDefinition,
    sprite_data: &SpriteData,
) -> Handle<Image> {
    textures.add(Image::new(
        Extent3d {
            width: sprite_definition.size_px.0,
            height: sprite_definition.size_px.1,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        sprite_data.data.to_vec(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    ))
}
