use std::ops::Add;

use crate::{buffer::BufferId, structs::Direction};

pub struct ScreenCursor {
    pub col: u16,
    pub row: u16,
}

impl ScreenCursor {
    pub fn is_top(&self) -> bool {
        self.row == 0
    }
    pub fn is_bottom(&self, num_rows: u16, offset: u16) -> bool {
        self.row == num_rows - 1 - offset
    }
}

/// Absolute position of the virtual cursor in the buffer
#[derive(Debug)]
pub struct BufferCursor {
    pub line: usize,
    pub col: usize,
}

impl BufferCursor {
    pub fn start() -> Self {
        Self { line: 0, col: 0 }
    }
}

// TODO: make this an enum with type Scratch, and
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct WindowId(pub usize);

impl Add<usize> for WindowId {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

#[derive(Debug)]
pub struct Window {
    pub id: WindowId,
    pub buffer_id: BufferId,
    pub cursor: BufferCursor,
    /// The absolute position of the first line that is visible
    pub scroll_offset: usize,
    /// Column offset
    pub col_offset: usize,
}

impl Window {
    pub fn new(id: WindowId, buffer_id: BufferId) -> Window {
        Window {
            id,
            buffer_id,
            cursor: BufferCursor::start(),
            scroll_offset: 0,
            col_offset: 0,
        }
    }
    pub fn move_cursor(&mut self, direction: Direction) {
        match direction {
            Direction::Up => {
                if self.cursor.line > 0 {
                    self.cursor.line -= 1;
                }
            }
            Direction::Down => {
                self.cursor.line += 1;
            }
            Direction::Left => {
                if self.cursor.col > 0 {
                    self.cursor.col -= 1;
                }
            }
            Direction::Right => {
                self.cursor.col += 1;
            }
        }
    }
    pub fn cursor_to_screen_coords(&self) -> ScreenCursor {
        let row = self.cursor.line - self.scroll_offset;
        let col = self.cursor.col;

        // TODO: should be fine, we should probably panic if this is out of bounds
        ScreenCursor {
            col: col as u16,
            row: row as u16,
        }
    }
}
