use std::io::Write;

use crossterm::{
    cursor, execute,
    terminal::{self, Clear},
};

use crate::{
    editor::{Editor, Mode},
    window::WindowId,
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
    current_tab: usize,
    tabs: Vec<Tab>,
}

#[derive(Debug)]
struct Tab {
    layout: Layout,
    focused_window: WindowId,
}

fn clear_screen(stdout: &mut impl Write) -> std::io::Result<()> {
    execute!(stdout, Clear(crossterm::terminal::ClearType::All))?;
    Ok(())
}

impl Screen {
    pub fn new() -> Self {
        Self {
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
    pub fn render(&self, stdout: &mut impl Write, editor: &Editor) -> anyhow::Result<()> {
        let (cols, rows) = terminal::size()?;
        clear_screen(stdout)?;
        execute!(stdout, cursor::MoveTo(0, 0))?;

        let current_window_id = self.current_window_id();
        let current_window = &editor.windows[current_window_id];

        // command buffer line in the bottom
        let num_lines = cols - 1;
        let offset = current_window.scroll_offset;
        let buffer = &editor.buffers[current_window.buffer_id];
        let buffer_num_lines = buffer.text.len();

        let start = offset;
        let end = (offset + num_lines as usize).min(buffer_num_lines);

        let lines = &buffer.text[start..end];

        let mut row = 0;
        // because we are rendering line numbers, our cursor is bugging
        for (line_number, line) in lines.iter().enumerate() {
            execute!(stdout, cursor::MoveTo(0, row))?;
            // print!("{:>3} ", line_number + offset + 1);
            let mut col = 0;
            for ch in line.chars() {
                if col >= cols {
                    break;
                }
                execute!(stdout, cursor::MoveTo(col, row))?;
                write!(stdout, "{}", ch)?;
                col += 1;
            }
            row += 1;
        }

        match editor.mode {
            Mode::Command => {
                execute!(stdout, cursor::MoveTo(0, rows - 1))?;
                print!(":{}", editor.command_buffer);
                stdout.flush()?;
            }
            _ => {
                let screen_cursor = current_window.cursor_to_screen_coords();
                execute!(stdout, cursor::MoveTo(screen_cursor.col, screen_cursor.row))?;
            }
        }

        Ok(())
    }
    pub fn paint(&self) {}
}
