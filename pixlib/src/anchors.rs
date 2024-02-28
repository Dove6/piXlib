use std::ops::Add;

use bevy::{
    math::Vec2,
    sprite::{Anchor, Sprite},
};

pub fn add_tuples<T: Add>(
    a: (T, T),
    b: (T, T),
) -> (<T as std::ops::Add>::Output, <T as std::ops::Add>::Output) {
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
