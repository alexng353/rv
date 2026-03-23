use std::{
    ops::{Index, IndexMut},
    path::PathBuf,
};

use crossterm::terminal;
use tracing::{info, instrument};

use crate::{
    buffer::{BufSource, Buffer, BufferId},
    errors::EditorError,
    screen::Screen,
    window::{BufferCursor, Direction, Window, WindowId},
};

#[derive(PartialEq, Debug)]
pub enum Mode {
    Normal,
    Insert,
    Command,
}

impl Index<BufferId> for Vec<Buffer> {
    type Output = Buffer;

    fn index(&self, id: BufferId) -> &Self::Output {
        &self[id.0]
    }
}

impl IndexMut<BufferId> for Vec<Buffer> {
    fn index_mut(&mut self, id: BufferId) -> &mut Self::Output {
        &mut self[id.0]
    }
}

#[derive(Debug)]
pub struct Editor {
    pub buffers: Vec<Buffer>,
    pub current_window: Window,
    pub command_buffer: String,
    pub mode: Mode,
}

pub trait Editing {
    fn backspace(&mut self);
    fn insert_char(&mut self, c: char);
    fn paste(&mut self);
    fn enter(&mut self);
}

impl Editing for Editor {
    fn backspace(&mut self) {
        let current_window = &mut self.current_window;
        let buffer = &mut self.buffers[current_window.buffer_id];
        buffer.dirty = true;

        let text = &mut buffer.text;

        if current_window.cursor.col != 0 {
            text[current_window.cursor.line].remove(current_window.cursor.col - 1);

            current_window.cursor.col -= 1;
        } else {
            let current_line = &text[current_window.cursor.line].clone();
            let prev_line = &mut text[current_window.cursor.line - 1];
            let next_col = prev_line.len();
            prev_line.push_str(current_line);
            buffer.text.remove(current_window.cursor.line);

            current_window.cursor.line -= 1;
            current_window.cursor.col = next_col;
        }
    }
    fn insert_char(&mut self, c: char) {
        let current_window = &self.current_window;
        let buffer = &mut self.buffers[current_window.buffer_id];
        buffer.dirty = true;
        let current_line = &mut buffer.text[current_window.cursor.line];

        info!(cursor_col = current_window.cursor.col);

        // possible for the cursor to be outside of the bounds of the line
        current_line.insert(current_window.cursor.col, c);
        self.current_window.cursor.col += 1;
    }
    fn paste(&mut self) {
        todo!()
    }
    fn enter(&mut self) {
        let current_window = &mut self.current_window;
        let buffer = &mut self.buffers[current_window.buffer_id];
        buffer.dirty = true;

        // this implementation is naive and does not work properly
        // TODO: fix it
        buffer
            .text
            .insert(current_window.cursor.line + 1, String::new());

        current_window.cursor.line += 1;
        current_window.cursor.col = 0;
    }
}

impl Editor {
    pub fn new() -> Self {
        let buf0 = Buffer::new_empty(0);
        Self {
            buffers: vec![buf0],
            current_window: Window {
                id: WindowId(0),
                buffer_id: BufferId(0),
                cursor: BufferCursor::start(),
                scroll_offset: 0,
                col_offset: 0,
            },
            command_buffer: String::with_capacity(1024),
            mode: Mode::Normal,
        }
    }
    fn max_id(&self) -> BufferId {
        self.buffers.last().map(|b| b.id).unwrap_or(BufferId(0))
        // self.buffers.iter().map(|b| b.id).max().unwrap_or(0)
    }
    fn new_buffer(&mut self, text: Vec<String>, source: BufSource) -> usize {
        let id = self.max_id().0 + 1;
        self.buffers.push(Buffer::new(id, text, source));
        id
    }
    fn open_buffer(&mut self, id: BufferId) -> Result<&mut Buffer, EditorError> {
        let buf = self
            .buffers
            .iter_mut()
            .find(|b| b.id == id)
            .ok_or(EditorError::BufferNotFound(id.0))?;

        Ok(buf)
    }
    pub fn open_file(&mut self, filepath: &PathBuf) -> Result<BufferId, EditorError> {
        let id = self.max_id() + 1;
        self.buffers.push(Buffer::new_from_file(id, filepath)?);
        Ok(id)
    }
    pub fn get_current_window(&self) -> &Window {
        // hypothetically impossible to error, but oh well
        &self.current_window
    }

    // handles cursor movement within the window, scrolling, etc.
    // window movement within the actual TUI is handled by the render_frame function
    pub fn move_cursor(&mut self, direction: Direction) -> anyhow::Result<()> {
        let screen_cursor = self.current_window.cursor_to_screen_coords();

        let current_offset = self.current_window.scroll_offset;
        let current_line = self.current_window.cursor.line;

        let current_col_offset = self.current_window.col_offset;
        let current_col = self.current_window.cursor.col;

        let (cols, rows) = terminal::size()?;

        let num_lines = self.buffers[self.current_window.buffer_id].text.len();

        // bug: if we scroll all the content off the screen, and scroll back up
        // the content doesn't come back
        let can_move = match direction {
            Direction::Up => {
                // buffer cursor is at the top of the screen
                // and we have scrolled
                if current_line - current_offset == 0 && self.current_window.scroll_offset > 0 {
                    self.current_window.scroll_offset -= 1;
                    // always scroll the screen cursor up if we are going to scroll the buffer cursor
                    // trust the ScreenCursor overflow protection
                    true
                } else {
                    !screen_cursor.is_top()
                }
            }
            Direction::Down => {
                // virtual cursor is at the bottom of the screen
                if (current_line - current_offset >= (rows - 1).into())
                    // and we're not at the bottom of the buffer
                    && self.current_window.scroll_offset < num_lines
                {
                    // move the scroll offset down one line
                    self.current_window.scroll_offset += 1;
                }

                // if we're at the bottom of the buffer, we can't move down
                if self.current_window.scroll_offset >= num_lines
                    && self.current_window.cursor_to_screen_coords().row >= rows - 1
                {
                    false
                } else {
                    true
                }
            }
            _ => true,
        };

        if can_move {
            self.current_window.move_cursor(direction);
        }

        Ok(())
    }
}
