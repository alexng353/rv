use tracing::info;

use crate::{editor::Editor, window::WindowId};

pub trait Editing {
    fn backspace(&mut self);
    fn delete(&mut self);
    fn insert_char(&mut self, window_id: WindowId, c: char);
    fn paste(&mut self, window_id: WindowId);
    fn enter(&mut self);
}

impl Editing for Editor {
    fn backspace(&mut self) {
        let current_window = &mut self.windows[self.tabs[self.current_tab].focused_window];
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

    fn delete(&mut self) {
        let current_window = &mut self.windows[self.tabs[self.current_tab].focused_window];
        let buffer = &mut self.buffers[current_window.buffer_id];
        buffer.dirty = true;

        let text = &mut buffer.text;
        if current_window.cursor.col < text[current_window.cursor.line].len() {
            text[current_window.cursor.line].remove(current_window.cursor.col);
        } else if current_window.cursor.line < text.len() - 1 {
            let current_line = &text[current_window.cursor.line].clone();

            let next_line = &mut text[current_window.cursor.line + 1];
            let next_col = next_line.len();
            next_line.push_str(current_line);
            buffer.text.remove(current_window.cursor.line);

            current_window.cursor.line += 1;
            current_window.cursor.col = next_col;
        }
    }

    fn insert_char(&mut self, window_id: WindowId, c: char) {
        let current_window = &mut self.windows[window_id];
        let buffer = &mut self.buffers[current_window.buffer_id];
        buffer.dirty = true;
        let current_line = &mut buffer.text[current_window.cursor.line];

        info!(cursor_col = current_window.cursor.col);

        // possible for the cursor to be outside of the bounds of the line
        current_line.insert(current_window.cursor.col, c);
        current_window.cursor.col += 1;
    }

    fn paste(&mut self, window_id: WindowId) {
        todo!()
    }

    fn enter(&mut self) {
        let current_window = &mut self.windows[self.tabs[self.current_tab].focused_window];
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
