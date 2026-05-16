use std::path::PathBuf;

use crossterm::terminal;
use tracing::info;

use crate::{
    buffer::{BufSource, Buffer, BufferId},
    command::{Axis, Command, Direction, Motion},
    config::ConfigRaw,
    editor::{Editing, Mode, tab::Tab},
    errors::EditorError,
    keymap::{Key, KeyCode, KeyMap, Modifiers, TrieMatch},
    layout::Layout,
    structs::Rect,
    window::{BufferCursor, ScrollState, Window, WindowId},
};
use anyhow::Result;

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
    keymap: KeyMap,
    pub should_quit: bool,
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
                scroll: ScrollState { row: 0, col: 0 },
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

    fn clamp_cursor(&mut self) {
        let window = &mut self.windows[self.tabs[self.current_tab].focused_window];
        let buffer = &self.buffers[window.buffer_id];
        window.cursor = window.cursor.clamp(buffer, &self.mode);
    }

    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        self.clamp_cursor();
    }

    fn dispatch(&mut self, command: Command) -> Result<()> {
        let window_id = self.current_window().id;
        match command {
            Command::InsertChar(c) => {
                self.insert_char(window_id, c);
            }
            Command::Insert(_) => {}
            Command::Move(motion) => self.motion(motion)?,
            Command::CycleTab(cycle) => {}
            Command::FocusWindow(direction) => {
                self.focus_window(direction)?;
            }
            Command::CycleBuffer(cycle) => {}
            Command::SetMode(mode) => {
                info!("Set mode to {}", mode);
                self.set_mode(mode);
            }
            Command::Exit(_) => self.should_quit = true,
            Command::Append => {
                self.mode = Mode::Insert;
                self.motion(Motion::Cursor(Direction::Right))?;
            }
            Command::AppendEol => {
                self.mode = Mode::Insert;
                self.motion(Motion::LineEnd)?;
            }
            Command::InsertLineStart => {
                self.mode = Mode::Insert;
                self.motion(Motion::LineStart)?;
            }
            Command::InsertZero => {
                self.mode = Mode::Insert;
                self.motion(Motion::LineZero)?;
            }
            Command::Scroll(scroll) => todo!(),
            Command::Split(axis) => self.split(axis)?,
        }
        Ok(())
    }

    fn focus_window(&mut self, direction: Direction) -> anyhow::Result<()> {
        let (cols, rows) = terminal::size()?;

        let tree = self.current_tab().walk(Rect {
            x: 0,
            y: 0,
            width: cols,
            height: rows - 1,
        });

        let window_id = self.current_window_id();
        let current_rect = tree
            .iter()
            .find(|(id, _)| *id == window_id)
            .map(|(_, rect)| *rect)
            .expect("WindowId does not exist");

        // TODO: make this cursor-aware for collisions
        match direction {
            Direction::Up => {
                for (id, candidate) in tree {
                    if candidate.y == current_rect.y.saturating_sub(current_rect.height) {
                        self.set_window(id);
                        break;
                    }
                }
            }
            Direction::Down => {
                for (id, candidate) in tree {
                    if candidate.y == current_rect.y + current_rect.height {
                        self.set_window(id);
                        break;
                    }
                }
            }
            Direction::Left => {
                for (id, candidate) in tree {
                    if candidate.x == current_rect.x.saturating_sub(current_rect.width) {
                        self.set_window(id);
                        break;
                    }
                }
            }
            Direction::Right => {
                // looking for the window that has the left edge == current.right
                for (id, candidate) in tree {
                    if candidate.x == current_rect.x + current_rect.width {
                        self.set_window(id);
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    fn write(&mut self) -> Result<()> {
        let window = self.current_window_id();
        let buffer = &mut self.buffers[self.windows[window].buffer_id];
        buffer.write()?;
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
                self.split(Axis::Horizontal)?;
            }
            "vsplit" => {
                self.split(Axis::Vertical)?;
            }
            "w" => {
                self.write()?;
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

    fn normalize(&self, key: Key) -> Key {
        if self.pending_keys.is_empty() {
            return key;
        }

        if self.pending_keys.len() == 1
            && (self.pending_keys[0]
                == (Key {
                    code: KeyCode::Char('w'),
                    mods: Modifiers::CONTROL,
                }))
        {
            return Key {
                code: key.code,
                mods: key.mods & !Modifiers::CONTROL,
            };
        }

        key
    }

    // TODO: handle <Enter> setting off the current command we found from TrieMatch::Prefix
    pub fn handle_key(&mut self, key: Key) -> Result<()> {
        let key = self.normalize(key);
        let window = self.current_window();
        let command = self.keymap.trie_match(&self.mode, &self.pending_keys, &key);
        info!("{}: {} {:?}", self.mode, key, command);

        match command {
            TrieMatch::None => {
                match self.mode {
                    Mode::Insert => match key.code {
                        KeyCode::Char(c) => self.insert_char(window.id, c),
                        KeyCode::Esc => self.mode = Mode::Normal,
                        KeyCode::Backspace => self.backspace(),
                        KeyCode::Enter => self.enter(),
                        KeyCode::Delete => self.delete(),
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
            TrieMatch::Word(command) => {
                self.dispatch(command)?;
                self.pending_keys.clear();
            }
        };

        Ok(())
    }

    pub fn split(&mut self, direction: Axis) -> anyhow::Result<()> {
        let (cols, rows) = terminal::size()?;
        let rect = Rect {
            x: 0,
            y: 0,
            height: rows,
            width: cols,
        };

        let scratch = self.new_buffer(vec!["".to_string()], BufSource::Scratch);
        let new_window = self.new_window(scratch);

        let tab = &mut self.tabs[self.current_tab];
        let current_focus = tab.focused_window;
        tab.layout
            .split_at(current_focus, new_window, direction, &rect);

        Ok(())
    }

    pub fn set_window(&mut self, window_id: WindowId) {
        self.current_tab_mut().set_window(window_id);
    }

    pub fn current_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.current_tab]
    }

    pub fn current_tab(&self) -> &Tab {
        &self.tabs[self.current_tab]
    }

    pub fn current_window(&self) -> &Window {
        &self.windows[self.current_tab().focused_window]
    }

    pub fn current_window_mut(&mut self) -> &mut Window {
        let focus = self.current_tab().focused_window;
        &mut self.windows[focus]
    }

    fn current_window_id(&self) -> WindowId {
        self.current_tab().focused_window
    }

    pub fn current_window_rect(&self, cols: u16, rows: u16) -> Rect {
        let tree = self.current_tab().walk(Rect {
            x: 0,
            y: 0,
            width: cols,
            height: rows - 1,
        });
        let window_id = self.current_window().id;
        tree.iter()
            .find(|(id, _)| *id == window_id)
            .map(|(_, rect)| *rect)
            .expect("WindowId does not exist")
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

    pub fn motion(&mut self, motion: Motion) -> anyhow::Result<()> {
        let rect = {
            let (cols, rows) = terminal::size()?;
            self.current_window_rect(cols, rows)
        };

        let window = {
            let window_id = self.current_window_id();
            &mut self.windows[window_id]
        };

        let buffer = &self.buffers[window.buffer_id];

        window.cursor = motion.target(&window.cursor, &buffer, &self.mode);
        window.scroll = adjust_scroll(&rect, &window.cursor, window.scroll, &buffer);

        info!("Cursor: {:?}, Scroll: {:?}", window.cursor, window.scroll);

        Ok(())
    }
}

fn adjust_scroll(
    rect: &Rect,
    cursor: &BufferCursor,
    scroll_state: ScrollState,
    buffer: &Buffer,
) -> ScrollState {
    let mut new = scroll_state;

    if cursor.line < new.row {
        new.row = cursor.line;
    } else if cursor.line >= new.row + rect.height as usize {
        new.row = cursor.line - rect.height as usize + 1;
    }

    if cursor.col < new.col {
        new.col = cursor.col;
    } else if cursor.col >= new.col + rect.width as usize {
        new.col = cursor.col - rect.width as usize + 1;
    }

    new.row = new.row.min(buffer.text.len().saturating_sub(1));
    new.col = new.col.min(
        buffer.text[cursor.line]
            .len()
            .saturating_sub(rect.width as usize),
    );

    new
}

#[cfg(test)]
mod adjust_scroll_tests {
    use super::*;
    const FILE: &str = include_str!("../../fixtures/lipsum.txt");

    #[test]
    fn scroll_down() {
        let rect = Rect {
            x: 0,
            y: 0,
            height: 10,
            width: 10,
        };
        let cursor = BufferCursor { line: 11, col: 0 };
        let buffer = Buffer::new_from_str(0, FILE);

        let scroll = adjust_scroll(&rect, &cursor, ScrollState { row: 10, col: 0 }, &buffer);
        assert_eq!(scroll.row, 1);
        assert_eq!(scroll.col, 0);
    }

    #[test]
    fn scroll_up() {
        let rect = Rect {
            x: 0,
            y: 0,
            height: 10,
            width: 10,
        };
        let cursor = BufferCursor { line: 0, col: 0 };
        let buffer = Buffer::new_from_str(0, FILE);

        let scroll = adjust_scroll(&rect, &cursor, ScrollState { row: 0, col: 0 }, &buffer);
        assert_eq!(scroll.row, 0);
        assert_eq!(scroll.col, 0);
    }

    #[test]
    fn scroll_right() {
        let rect = Rect {
            x: 0,
            y: 0,
            height: 10,
            width: 10,
        };
        let cursor = BufferCursor { line: 0, col: 11 };
        let buffer = Buffer::new_from_str(0, FILE);

        let scroll = adjust_scroll(&rect, &cursor, ScrollState { row: 0, col: 0 }, &buffer);
        assert_eq!(scroll.row, 0);
        assert_eq!(scroll.col, 1);
    }
}
