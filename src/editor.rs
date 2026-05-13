use std::{
    ops::{Index, IndexMut},
    path::PathBuf,
};

use crossterm::terminal;
use tracing::info;

use crate::{
    buffer::{BufSource, Buffer, BufferId},
    config::ConfigRaw,
    errors::EditorError,
    keymap::{Command, Key, KeyCode, KeyMap, TrieMatch},
    layout::Layout,
    screen::SplitDirection,
    structs::{Direction, Rect},
    window::{BufferCursor, Window, WindowId},
};
use anyhow::Result;

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Mode {
    Normal,
    Insert,
    Command,
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Normal => write!(f, "Normal"),
            Mode::Insert => write!(f, "Insert"),
            Mode::Command => write!(f, "Command"),
        }
    }
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

// TODO: add editor.message (echo output)
// TODO: buffers & windows should be private members
#[derive(Debug)]
pub struct Editor {
    pub buffers: Vec<Buffer>,
    pub windows: Vec<Window>,
    pub command_buffer: String,
    pub mode: Mode,
    pub tabs: Vec<Tab>,
    pub current_tab: usize,
    pending_keys: Vec<Key>,
    pub keymap: KeyMap,
    pub should_quit: bool,
}

#[derive(Debug)]
pub struct Tab {
    layout: Layout,
    focused_window: WindowId,
}

impl Tab {
    pub fn walk(&self, rect: Rect) -> Vec<(WindowId, Rect)> {
        self.layout.walk(rect)
    }
}

impl Editor {
    pub fn new(raw_config: &ConfigRaw) -> anyhow::Result<Self> {
        let buf0 = Buffer::new_empty(0);
        Ok(Self {
            buffers: vec![buf0],
            windows: vec![Window {
                id: WindowId(0),
                buffer_id: BufferId(0),
                cursor: BufferCursor::start(),
                scroll_offset: 0,
                col_offset: 0,
            }],
            command_buffer: String::with_capacity(1024),
            mode: Mode::Normal,
            tabs: vec![Tab {
                layout: Layout::Leaf(WindowId(0)),
                focused_window: WindowId(0),
            }],
            current_tab: 0,
            pending_keys: vec![],
            keymap: KeyMap::new(
                &raw_config.keymaps.normal,
                &raw_config.keymaps.insert,
                &raw_config.keymaps.command,
            )?,
            should_quit: false,
        })
    }

    fn dispatch(&mut self, command: Command) -> Result<()> {
        let window = self.current_window();
        match command {
            Command::InsertChar(c) => {
                self.insert_char(window.id, c);
            }
            Command::Insert(_) => {}
            Command::MoveCursor(direction) => self.move_cursor(window.id, direction)?,
            Command::CycleTab(cycle) => {}
            Command::FocusWindow(direction) => {}
            Command::CycleBuffer(cycle) => {}
            Command::SetMode(mode) => {
                info!("Set mode to {}", mode);
                self.mode = mode
            }
            Command::Exit(_) => self.should_quit = true,
        }
        Ok(())
    }

    // TODO: actual command registry
    fn execute_command(&mut self) -> Result<()> {
        info!("command_buffer: {}", self.command_buffer);
        match self.command_buffer.as_str() {
            "q" => {
                self.should_quit = true;
            }
            "split" => {
                self.split(SplitDirection::Horizontal)?;
            }
            "vsplit" => {
                self.split(SplitDirection::Vertical)?;
            }
            _ => {}
        };

        if let Some(command) = self.command_buffer.split_once(' ') {
            if command.0 == "e" {
                let id = self.open_file(&PathBuf::from(command.1))?;
                let window = self.current_window_mut();
                window.buffer_id = id;
            }
        }

        self.command_buffer.clear();
        self.mode = Mode::Normal;

        Ok(())
    }

    // TODO: handle <Enter> setting off the current command we found from TrieMatch::Prefix
    pub fn handle_key(&mut self, key: Key) -> Result<()> {
        let window = self.current_window();
        let command = self.keymap.trie_match(&self.mode, &self.pending_keys, &key);
        info!("{}: {} {:?}", self.mode, key, command);

        match command {
            TrieMatch::None => {
                match self.mode {
                    Mode::Insert => match key.code {
                        KeyCode::Char(c) => self.insert_char(window.id, c),
                        KeyCode::Esc => self.mode = Mode::Normal,
                        KeyCode::Backspace => self.backspace(window.id),
                        KeyCode::Enter => self.enter(window.id),
                        _ => {}
                    },
                    Mode::Normal => {
                        // No fallback in Normal Mode, just clear pending
                    }
                    Mode::Command => match key.code {
                        KeyCode::Char(c) => {
                            self.command_buffer.push(c);
                        }
                        KeyCode::Backspace => {
                            if self.command_buffer.is_empty() {
                                self.mode = Mode::Normal;
                            } else {
                                self.command_buffer.pop();
                            }
                        }
                        KeyCode::Enter => self.execute_command()?,
                        // TODO: move cursor within command buffer
                        KeyCode::Left => {}
                        KeyCode::Right => {}
                        KeyCode::Up => {}
                        KeyCode::Down => {}
                        KeyCode::Home => {}
                        KeyCode::End => {}

                        KeyCode::PageUp => {}
                        KeyCode::PageDown => {}
                        // TODO: insert a tab char
                        KeyCode::Tab => todo!(),
                        KeyCode::BackTab => {}
                        // TODO: forward delete
                        KeyCode::Delete => {}
                        _ => {}
                    },
                }
                self.pending_keys.clear();
            }
            TrieMatch::Prefix(_) => self.pending_keys.push(key),
            TrieMatch::Word(command) => self.dispatch(command)?,
        };

        Ok(())
    }

    pub fn split(&mut self, direction: SplitDirection) -> anyhow::Result<()> {
        let (cols, rows) = terminal::size()?;
        let rect = Rect {
            x: 0,
            y: 0,
            height: rows,
            width: cols,
        };

        let scratch = self.new_buffer(vec![], BufSource::Scratch);
        let new_window = self.new_window(scratch);

        let tab = &mut self.tabs[self.current_tab];
        let current_focus = tab.focused_window;
        tab.layout
            .split_at(current_focus, new_window, direction, &rect);

        Ok(())
    }

    pub fn current_tab(&self) -> &Tab {
        &self.tabs[self.current_tab]
    }

    pub fn current_window(&self) -> &Window {
        &self.windows[self.current_tab().focused_window]
    }
    fn current_window_mut(&mut self) -> &mut Window {
        let focus = self.current_tab().focused_window;
        &mut self.windows[focus]
    }

    fn max_buffer_id(&self) -> BufferId {
        self.buffers.last().map(|b| b.id).unwrap_or(BufferId(0))
    }

    fn max_window_id(&self) -> WindowId {
        self.windows.last().map(|w| w.id).unwrap_or(WindowId(0))
    }

    pub fn new_window(&mut self, buffer_id: BufferId) -> WindowId {
        let id = self.max_window_id() + 1;
        self.windows.push(Window::new(id, buffer_id));
        id
    }

    pub fn new_buffer(&mut self, text: Vec<String>, source: BufSource) -> BufferId {
        let id = self.max_buffer_id() + 1;
        self.buffers.push(Buffer::new(id.0, text, source));
        id
    }

    fn get_buffer(&mut self, id: BufferId) -> Result<&mut Buffer, EditorError> {
        let buf = self
            .buffers
            .iter_mut()
            .find(|b| b.id == id)
            .ok_or(EditorError::BufferNotFound(id.0))?;

        Ok(buf)
    }

    pub fn open_file(&mut self, filepath: &PathBuf) -> Result<BufferId, EditorError> {
        let id = self.max_buffer_id() + 1;
        self.buffers.push(Buffer::new_from_file(id, filepath)?);
        Ok(id)
    }

    // handles cursor movement within the window, scrolling, etc.
    // window movement within the actual TUI is handled by the render_frame function
    pub fn move_cursor(&mut self, window_id: WindowId, direction: Direction) -> anyhow::Result<()> {
        let current_window = &mut self.windows[window_id];
        let screen_cursor = current_window.cursor_to_screen_coords();

        let current_offset = current_window.scroll_offset;
        let current_line = current_window.cursor.line;

        let current_col_offset = current_window.col_offset;
        let current_col = current_window.cursor.col;

        let (cols, rows) = terminal::size()?;

        let num_lines = self.buffers[current_window.buffer_id].text.len();

        // bug: if we scroll all the content off the screen, and scroll back up
        // the content doesn't come back
        let can_move = match direction {
            Direction::Up => {
                // buffer cursor is at the top of the screen
                // and we have scrolled
                if current_line - current_offset == 0 && current_window.scroll_offset > 0 {
                    current_window.scroll_offset -= 1;
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
                    && current_window.scroll_offset < num_lines
                {
                    // move the scroll offset down one line
                    current_window.scroll_offset += 1;
                }

                // if we're at the bottom of the buffer, we can't move down
                if current_window.scroll_offset >= num_lines
                    && current_window.cursor_to_screen_coords().row >= rows - 1
                {
                    false
                } else {
                    true
                }
            }
            _ => true,
        };

        if can_move {
            current_window.move_cursor(direction);
        }

        Ok(())
    }
}

pub trait Editing {
    fn backspace(&mut self, window_id: WindowId);
    fn insert_char(&mut self, window_id: WindowId, c: char);
    fn paste(&mut self, window_id: WindowId);
    fn enter(&mut self, window_id: WindowId);
}

impl Editing for Editor {
    fn backspace(&mut self, window_id: WindowId) {
        let current_window = &mut self.windows[window_id];
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

    fn enter(&mut self, window_id: WindowId) {
        let current_window = &mut self.windows[window_id];
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
