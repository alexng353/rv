use crate::screen::SplitDirection;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub fn split(&self, direction: SplitDirection, split_at: u16) -> (Rect, Rect) {
        match direction {
            SplitDirection::Vertical => {
                let left_width = std::cmp::min(self.width, split_at);
                (
                    // left
                    Rect {
                        x: self.x,
                        y: self.y,
                        width: left_width,
                        height: self.height,
                    },
                    // right
                    Rect {
                        x: self.x + left_width,
                        y: self.y,
                        // whatever's left of self.width, bounded at 0
                        // subtraction safe with no underflow because we clamped the left_width
                        // value to be <= self.width
                        width: self.width - left_width,
                        height: self.height,
                    },
                )
            }
            SplitDirection::Horizontal => {
                let top_height = std::cmp::min(self.height, split_at);
                (
                    // top
                    Rect {
                        x: self.x,
                        y: self.y,
                        width: self.width,
                        height: top_height,
                    },
                    // bottom
                    Rect {
                        x: self.x,
                        y: self.y + top_height,
                        width: self.width,
                        // same reasoning as above
                        height: self.height - top_height,
                    },
                )
            }
        }
    }
}
