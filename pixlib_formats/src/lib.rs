pub mod compression_algorithms;
pub mod file_formats;

#[allow(clippy::assertions_on_constants)]
const _: () = assert!(usize::BITS >= u32::BITS);

#[derive(Debug, Clone, Copy, Default)]
pub struct Rect {
    pub top_left_x: isize,
    pub top_left_y: isize,
    pub bottom_right_x: isize,
    pub bottom_right_y: isize,
}

impl Rect {
    pub fn from(position: (isize, isize), size: (usize, usize)) -> Self {
        Self {
            top_left_x: position.0,
            top_left_y: position.1,
            bottom_right_x: position.0 + size.0 as isize,
            bottom_right_y: position.1 + size.1 as isize,
        }
    }

    pub fn intersect(&self, other: &Self) -> Option<Self> {
        let intersection = Self {
            top_left_x: self.top_left_x.max(other.top_left_x),
            top_left_y: self.top_left_y.max(other.top_left_y),
            bottom_right_x: self.bottom_right_x.min(other.bottom_right_x),
            bottom_right_y: self.bottom_right_y.min(other.bottom_right_y),
        };
        if intersection.bottom_right_x <= intersection.top_left_x
            || intersection.bottom_right_y <= intersection.top_left_y
        {
            None
        } else {
            Some(intersection)
        }
    }

    pub fn has_inside(&self, x: isize, y: isize) -> bool {
        x.clamp(self.top_left_x, self.bottom_right_x) == x
            && y.clamp(self.top_left_y, self.bottom_right_y) == y
    }

    pub fn get_width(&self) -> usize {
        (self.bottom_right_x - self.top_left_x) as usize
    }

    pub fn get_height(&self) -> usize {
        (self.bottom_right_y - self.top_left_y) as usize
    }

    pub fn get_center(&self) -> (isize, isize) {
        (
            self.top_left_x + self.get_width() as isize / 2,
            self.top_left_y + self.get_height() as isize / 2,
        )
    }
}

impl From<(isize, isize, isize, isize)> for Rect {
    fn from(value: (isize, isize, isize, isize)) -> Self {
        Self {
            top_left_x: value.0,
            top_left_y: value.1,
            bottom_right_x: value.2,
            bottom_right_y: value.3,
        }
    }
}
