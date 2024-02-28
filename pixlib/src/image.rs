use std::iter;

use bevy::{
    asset::Assets,
    ecs::system::ResMut,
    prelude::default,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureFormat},
        texture::Image,
    },
    sprite::{Anchor, Sprite, SpriteBundle},
    transform::components::Transform,
};
use pixlib_formats::{
    compression_algorithms::{lzw2::decode_lzw2, rle::decode_rle},
    file_formats::{img::ImgFile, ColorFormat, CompressionType, ImageData},
};

pub fn img_file_to_sprite_bundle(
    img_file: &ImgFile,
    textures: &mut ResMut<Assets<Image>>,
) -> SpriteBundle {
    let transform = Transform::from_xyz(
        img_file.header.x_position_px as f32,
        -img_file.header.y_position_px as f32,
        0.0,
    );
    let image = image_data_to_image(
        &img_file.image_data,
        (img_file.header.width_px, img_file.header.height_px),
        img_file.header.color_format,
        img_file.header.compression_type,
        img_file.header.alpha_size_bytes > 0,
    );
    let texture = textures.add(image);

    SpriteBundle {
        texture,
        sprite: Sprite {
            anchor: Anchor::TopLeft,
            ..default()
        },
        transform,
        ..default()
    }
}

pub fn image_data_to_image(
    image_data: &ImageData,
    image_size_px: (u32, u32),
    color_format: ColorFormat,
    compression_type: CompressionType,
    has_alpha: bool,
) -> Image {
    let color_data = match compression_type {
        CompressionType::None => image_data.color.to_owned(),
        CompressionType::Rle => decode_rle(image_data.color, 2),
        CompressionType::Lzw2 => decode_lzw2(image_data.color),
        CompressionType::RleInLzw2 => decode_rle(&decode_lzw2(image_data.color), 2),
        _ => panic!(),
    };
    let alpha_data = match compression_type {
        _ if !has_alpha => vec![],
        CompressionType::None => image_data.alpha.to_owned(),
        CompressionType::Rle => decode_rle(image_data.alpha, 1),
        CompressionType::Lzw2 | CompressionType::Jpeg => decode_lzw2(image_data.alpha),
        CompressionType::RleInLzw2 => decode_rle(&decode_lzw2(image_data.alpha), 1),
    };
    let color_converter = match color_format {
        ColorFormat::Rgb565 => rgb565_to_rgb888,
        ColorFormat::Rgb555 => rgb555_to_rgb888,
    };
    let converted_image = color_data
        .chunks_exact(2)
        .map(|rgb565| rgb565.try_into().unwrap())
        .map(color_converter)
        .zip(alpha_data.iter().chain(iter::repeat(&255)))
        .flat_map(|([r, g, b], a)| [r, g, b, *a])
        .collect();
    Image::new(
        Extent3d {
            width: image_size_px.0,
            height: image_size_px.1,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        converted_image,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

fn rgb565_to_rgb888(rgb565: [u8; 2]) -> [u8; 3] {
    let rgb565 = u16::from_le_bytes(rgb565);
    let r5 = (rgb565 >> 11) & 0x1f;
    let g6 = (rgb565 >> 5) & 0x3f;
    let b5 = rgb565 & 0x1f;
    [
        (r5 * 255 / 31).try_into().unwrap(),
        (g6 * 255 / 63).try_into().unwrap(),
        (b5 * 255 / 31).try_into().unwrap(),
    ]
}

fn rgb555_to_rgb888(rgb555: [u8; 2]) -> [u8; 3] {
    let rgb555 = u16::from_le_bytes(rgb555);
    let r5 = (rgb555 >> 10) & 0x1f;
    let g5 = (rgb555 >> 5) & 0x1f;
    let b5 = rgb555 & 0x1f;
    [
        (r5 * 255 / 31).try_into().unwrap(),
        (g5 * 255 / 31).try_into().unwrap(),
        (b5 * 255 / 31).try_into().unwrap(),
    ]
}
