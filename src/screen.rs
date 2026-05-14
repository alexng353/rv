use std::io::Write;

use crossterm::{
    cursor, queue,
    style::Print,
    terminal::{self, Clear, ClearType},
};

use crate::{
    buffer::Buffer,
    editor::{Editor, Mode},
    structs::Rect,
    window::{ScreenCursor, Window},
};

// TODO: move tabs + current tab + layouts, etc into editor instead, alongside Windows and Buffers
// because it makes more sense, I guess
#[derive(Debug)]
pub struct Screen {
    framebuffer: Vec<u8>,
}

#[allow(dead_code)]
trait TakeChars {
    fn take_chars(&self, n: usize) -> &Self;

    fn skip_chars(&self, n: usize) -> &Self;

    fn slice_chars(&self, skip: usize, take: usize) -> &Self;
}

impl TakeChars for str {
    fn take_chars(&self, n: usize) -> &Self {
        let end = self
            .char_indices()
            .nth(n)
            .map(|(i, _)| i)
            .unwrap_or(self.len());
        &self[0..end]
    }

    fn skip_chars(&self, n: usize) -> &Self {
        let end = self
            .char_indices()
            .nth(n)
            .map(|(i, _)| i)
            .unwrap_or(self.len());
        &self[end..]
    }

    fn slice_chars(&self, skip: usize, take: usize) -> &Self {
        let start = self
            .char_indices()
            .nth(skip)
            .map(|(i, _)| i)
            .unwrap_or(self.len());
        let end = self
            .char_indices()
            .nth(take)
            .map(|(i, _)| i)
            .unwrap_or(self.len());
        &self[start..end]
    }
}

// TODO: render horizontal scrolling (lol)
fn render_window(
    rect: Rect,
    window: &Window,
    buffer: &Buffer,
    focused: bool,
    framebuf: &mut Vec<u8>,
) -> anyhow::Result<()> {
    let num_lines = rect.height;
    let offset = window.scroll.row;
    let buffer_num_lines = buffer.text.len();

    let start = offset;
    let end = (offset + num_lines as usize).min(buffer_num_lines);

    let col_offset = window.scroll.col;
    let col_start = col_offset;
    let col_end = col_offset + rect.width as usize;

    let lines = &buffer.text[start..end];

    for i in 0..(rect.height - 1) {
        let line = lines.get(i as usize);
        let data = line
            .map(|l| l.slice_chars(col_start, col_end))
            .unwrap_or("");
        queue!(
            framebuf,
            cursor::MoveTo(rect.x, rect.y + i),
            Print(format!("{}", data)),
            // Print(format!("{} {}", i, data)),
            Clear(ClearType::UntilNewLine)
        )?;
    }

    // Chin bar
    // TODO: give it a different color
    let bufname = buffer.name()?;
    queue!(
        framebuf,
        cursor::MoveTo(rect.x, rect.height + rect.y - 1),
        Print(format!("{}", bufname)), // TODO: make this content a buffer, and give it an extra [ + ] if the
        // Print(format!("{} {}", rect.height + rect.y - 1, bufname)), // TODO: make this content a buffer, and give it an extra [ + ] if the
        // buffer is modified
        Clear(ClearType::UntilNewLine)
    )?;

    Ok(())
}

impl Screen {
    pub fn new() -> Self {
        Self {
            framebuffer: Vec::with_capacity(64 * 1024), // 64 kilobyte framebuffer
        }
    }

    fn build_frame(
        &mut self,
        editor: &Editor,
        cols: u16,
        rows: u16,
    ) -> anyhow::Result<ScreenCursor> {
        let screen_rect = Rect {
            x: 0,
            y: 0,
            width: cols,
            height: rows - 1, // accounts for statusline
        };
        let tree = editor.current_tab().walk(screen_rect);
        for (window_id, rect) in tree {
            let window = &editor.windows[window_id];
            render_window(
                rect,
                window,
                &editor.buffers[window.buffer_id],
                true,
                &mut self.framebuffer,
            )?;
        }

        // statusline
        queue!(
            self.framebuffer,
            cursor::MoveTo(0, rows - 1),
            Print(format!("{}", editor.mode)),
            Clear(ClearType::UntilNewLine)
        )?;

        let rect = editor.current_window_rect(cols, rows);
        Ok(editor.current_window().cursor_to_screen_coords(rect))
    }

    pub fn render(&mut self, stdout: &mut impl Write, editor: &Editor) -> anyhow::Result<()> {
        let (cols, rows) = terminal::size()?;
        self.framebuffer.clear();
        queue!(self.framebuffer, cursor::Hide, cursor::MoveTo(0, 0))?;

        let screen_cursor = self.build_frame(editor, cols, rows)?;

        match editor.mode {
            Mode::Command => {
                let command_buffer = editor.command_buffer.take_chars(cols as usize);

                queue!(
                    self.framebuffer,
                    cursor::MoveTo(0, rows - 1),
                    Print(format!(":{}", command_buffer)),
                    Clear(ClearType::UntilNewLine)
                )?;
            }
            _ => {
                queue!(
                    self.framebuffer,
                    cursor::MoveTo(screen_cursor.col, screen_cursor.row)
                )?;
            }
        }
        queue!(self.framebuffer, cursor::Show)?;

        stdout.write_all(&self.framebuffer)?;
        stdout.flush()?;

        Ok(())
    }
}
