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
pub struct BufferCursor {
    pub line: usize,
    pub col: usize,
}

impl BufferCursor {
    pub fn start() -> Self {
        Self { line: 0, col: 0 }
    }
}

pub struct Window {
    pub buffer_id: usize,
    pub cursor: BufferCursor,
    /// The absolute position of the first line that is visible
    pub scroll_offset: usize,
    /// Column offset
    pub col_offset: usize,
}

impl Window {
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

#[derive(PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
