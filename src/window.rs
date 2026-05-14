use std::ops::{Add, Index, IndexMut};

use tracing::info;

use crate::{
    buffer::{Buffer, BufferId},
    command::Direction,
    editor::Mode,
    structs::Rect,
};

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
    pub fn is_left(&self) -> bool {
        self.col == 0
    }
    pub fn is_right(&self, cols: u16, offset: u16) -> bool {
        self.col == cols - 1 - offset
    }
}

/// Absolute position of the virtual cursor in the buffer
#[derive(Debug, Copy, Clone)]
pub struct BufferCursor {
    pub line: usize,
    pub col: usize,
}

impl BufferCursor {
    pub fn start() -> Self {
        Self { line: 0, col: 0 }
    }

    /// Does not clamp in-place, you must set the cursor to the clamped value
    pub fn clamp(&self, buffer: &Buffer, mode: &Mode) -> BufferCursor {
        if mode != &Mode::Normal {
            return *self;
        }

        let line_length = buffer.text[self.line.min(buffer.text.len() - 1)].len();
        let num_lines = buffer.text.len();

        BufferCursor {
            col: self.col.min(line_length.saturating_sub(1)),
            line: self.line.min(num_lines - 1),
        }
    }
}

// TODO: make this an enum with type Scratch, and
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct WindowId(pub usize);

impl Index<WindowId> for Vec<Window> {
    type Output = Window;

    fn index(&self, id: WindowId) -> &Self::Output {
        &self[id.0]
    }
}

impl IndexMut<WindowId> for Vec<Window> {
    fn index_mut(&mut self, id: WindowId) -> &mut Self::Output {
        &mut self[id.0]
    }
}

impl Add<usize> for WindowId {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ScrollState {
    pub row: usize,
    pub col: usize,
}

#[derive(Debug, Copy, Clone)]
pub struct Window {
    pub id: WindowId,
    pub buffer_id: BufferId,
    pub cursor: BufferCursor,
    pub scroll: ScrollState,
}

impl Window {
    pub fn new(id: WindowId, buffer_id: BufferId) -> Window {
        Window {
            id,
            buffer_id,
            cursor: BufferCursor::start(),
            scroll: ScrollState { row: 0, col: 0 },
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
                info!("window cursor col = {}", self.cursor.col);
            }
        }
    }

    /// Unbounded positive scroll, bounded negative scrolling
    pub fn scroll(&mut self, direction: Direction) {
        match direction {
            Direction::Up => {
                if self.scroll.row > 0 {
                    self.scroll.row -= 1
                }
            }
            Direction::Down => self.scroll.row += 1,
            Direction::Left => {
                if self.scroll.col > 0 {
                    self.scroll.col -= 1
                }
            }
            Direction::Right => self.scroll.col += 1,
        }
    }

    pub fn cursor_to_screen_coords(&self, rect: Rect) -> ScreenCursor {
        let row = (rect.y as usize) + self.cursor.line - self.scroll.row;
        let col = (rect.x as usize) + self.cursor.col - self.scroll.col;

        ScreenCursor {
            col: col as u16,
            row: row as u16,
        }
    }
}
