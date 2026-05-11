use std::io::Write;

use crossterm::{
    cursor, queue,
    style::Print,
    terminal::{self, Clear, ClearType},
};

use crate::{
    buffer::Buffer,
    editor::{Editor, Mode},
    layout::Layout,
    structs::Rect,
    window::{ScreenCursor, Window, WindowId},
};

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum SplitDirection {
    /// The screen split runs horizontally
    Horizontal,
    /// The screen split runs vertically
    Vertical,
}

#[derive(Debug)]
pub struct Screen {
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

fn render_window(
    rect: Rect,
    window: &Window,
    buffer: &Buffer,
    focused: bool,
    framebuf: &mut Vec<u8>,
) -> anyhow::Result<()> {
    let num_lines = rect.height;
    let offset = window.scroll_offset;
    let buffer_num_lines = buffer.text.len();

    let start = offset;
    let end = (offset + num_lines as usize).min(buffer_num_lines);

    let lines = &buffer.text[start..end];

    for i in 0..(rect.height - 1) {
        let line = lines.get(i as usize);
        let data = line
            .map(|l| l.take_chars(rect.width as usize))
            .unwrap_or("");
        queue!(
            framebuf,
            cursor::MoveTo(rect.x, rect.y + i),
            Print(format!("{} {}", i, data)),
            Clear(ClearType::UntilNewLine)
        )?;
    }

    // Chin bar
    // TODO: give it a different color
    let bufname = buffer.name()?;
    queue!(
        framebuf,
        cursor::MoveTo(rect.x, rect.height + rect.y - 1),
        Print(format!("{} {}", rect.height + rect.y - 1, bufname)), // TODO: make this content a buffer, and give it an extra [ + ] if the
        // buffer is modified
        Clear(ClearType::UntilNewLine)
    )?;

    Ok(())
}

impl Screen {
    pub fn new() -> Self {
        Self {
            framebuffer: Vec::with_capacity(64 * 1024), // 64 kilobyte framebuffer
            tabs: vec![Tab {
                layout: Layout::Leaf(WindowId(0)),
                focused_window: WindowId(0),
            }],
            current_tab: 0,
        }
    }

    // TODO: this code is wrong, we basically need to find the Window within
    // the current Layout then we need to split that window, which should be a Layout::Leaf
    pub fn split(&mut self, new_window: WindowId, direction: SplitDirection) -> anyhow::Result<()> {
        let tab = &mut self.tabs[self.current_tab];

        let (cols, rows) = terminal::size()?;
        let rect = Rect {
            x: 0,
            y: 0,
            height: rows,
            width: cols,
        };

        let current_focus = tab.focused_window;

        tab.layout
            .split_at(current_focus, new_window, direction, &rect);

        Ok(())
    }

    pub fn current_window_id(&self) -> WindowId {
        self.tabs[self.current_tab].focused_window
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
        let tree = self.tabs[self.current_tab].layout.walk(screen_rect);
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

        let current_window_id = self.current_window_id();
        let current_window = &editor.windows[current_window_id];

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
}
