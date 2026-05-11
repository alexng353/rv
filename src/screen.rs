use std::io::Write;

use crossterm::{
    cursor, execute, queue,
    style::Print,
    terminal::{self, Clear, ClearType},
};

use crate::{
    editor::{Editor, Mode},
    window::{ScreenCursor, WindowId},
};

#[derive(PartialEq, Debug)]
enum SplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug)]
enum Layout {
    Leaf(WindowId),
    Split {
        direction: SplitDirection,
        children: Vec<Layout>,
        split_at: u16,
    },
}

#[derive(Debug)]
pub struct Screen {
    frame: u64,
    framebuffer: Vec<u8>,
    current_tab: usize,
    tabs: Vec<Tab>,
}

#[derive(Debug)]
struct Tab {
    layout: Layout,
    focused_window: WindowId,
}

trait TakeChars {
    fn take_chars(&self, n: usize) -> &Self;
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
}

impl Screen {
    pub fn new() -> Self {
        Self {
            frame: 0,
            framebuffer: vec![0; 64 * 1024], // 64 kilobyte framebuffer
            tabs: vec![Tab {
                layout: Layout::Leaf(WindowId(0)),
                focused_window: WindowId(0),
            }],
            current_tab: 0,
        }
    }
    pub fn current_window_id(&self) -> WindowId {
        self.tabs[self.current_tab].focused_window
    }

    pub fn build_frame(
        &mut self,
        editor: &Editor,
        cols: u16,
        rows: u16,
    ) -> anyhow::Result<ScreenCursor> {
        let current_window_id = self.current_window_id();
        let current_window = &editor.windows[current_window_id];

        let num_lines = rows;
        let offset = current_window.scroll_offset;
        let buffer = &editor.buffers[current_window.buffer_id];
        let buffer_num_lines = buffer.text.len();

        let start = offset;
        let end = (offset + num_lines as usize).min(buffer_num_lines);

        let lines = &buffer.text[start..end];

        for i in 0..=rows {
            let line = lines.get(i as usize);
            let data = line.map(|l| l.take_chars(cols as usize)).unwrap_or("");
            queue!(
                self.framebuffer,
                cursor::MoveTo(0, i),
                Print(format!("{}", data)),
                Clear(ClearType::UntilNewLine)
            )?;
        }
        Ok(current_window.cursor_to_screen_coords())
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
    pub fn paint(&self) {}
}
