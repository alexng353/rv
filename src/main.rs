// use clap::Parser;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{self, Clear, disable_raw_mode, enable_raw_mode},
};
use std::{io::Write, path::PathBuf};
use thiserror::Error;

// /// A toy text editor in Rust
// #[derive(Parser, Debug)]
// #[command(version, about, long_about = None)]
// struct Args {
//     /// The file to edit
//     file: Option<String>,
// }

#[derive(PartialEq)]
enum Mode {
    Normal,
    Insert,
    Command,
}

enum BufSource {
    Scratch,
    File(PathBuf),
    // Terminal
}

struct Buffer {
    pub id: usize,
    pub text: Vec<String>,
    pub source: BufSource,
    pub dirty: bool,
}

impl Buffer {
    fn new(id: usize, text: Vec<String>, source: BufSource) -> Self {
        Self {
            id,
            text,
            source,
            dirty: false,
        }
    }
    fn new_empty(id: usize) -> Self {
        Self {
            id,
            text: Vec::new(),
            source: BufSource::Scratch,
            dirty: false,
        }
    }
    fn new_from_file(id: usize, filepath: &PathBuf) -> Result<Self, std::io::Error> {
        let text = std::fs::read_to_string(filepath)?;
        Ok(Self {
            id,
            text: text.lines().map(|s| s.to_string()).collect(),
            source: BufSource::File(filepath.clone()),
            dirty: false,
        })
    }
}

/// Absolute position of the virtual cursor in the buffer
struct BufferCursor {
    line: usize,
    col: usize,
}
impl BufferCursor {
    fn start() -> Self {
        Self { line: 0, col: 0 }
    }
}

struct ScreenCursor {
    col: u16,
    row: u16,
}

struct Window {
    buffer_id: usize,
    cursor: BufferCursor,
    /// The absolute position of the first line that is visible
    scroll_offset: usize,
}

#[derive(PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Window {
    fn move_cursor(&mut self, direction: Direction) {
        match direction {
            Direction::Up => {
                self.cursor.line -= 1;
            }
            Direction::Down => {
                self.cursor.line += 1;
            }
            Direction::Left => {
                self.cursor.col -= 1;
            }
            Direction::Right => {
                self.cursor.col += 1;
            }
        }
    }
    fn cursor_to_screen_coords(&self) -> ScreenCursor {
        let row = self.cursor.line - self.scroll_offset;
        let col = self.cursor.col;

        // TODO: should be fine, we should probably panic if this is out of bounds
        ScreenCursor {
            col: col as u16,
            row: row as u16,
        }
    }
}

struct Editor {
    buffers: Vec<Buffer>,
    current_window: Window,
    command_buffer: String,
    mode: Mode,
}

impl Editor {
    fn new() -> Self {
        let buf0 = Buffer::new_empty(0);
        Self {
            buffers: vec![buf0],
            current_window: Window {
                buffer_id: 0,
                cursor: BufferCursor::start(),
                scroll_offset: 0,
            },
            command_buffer: String::with_capacity(1024),
            mode: Mode::Normal,
        }
    }
    fn max_id(&self) -> usize {
        self.buffers.last().map(|b| b.id).unwrap_or(0)
        // self.buffers.iter().map(|b| b.id).max().unwrap_or(0)
    }
    fn new_buffer(&mut self, text: Vec<String>, source: BufSource) -> usize {
        let id = self.max_id() + 1;
        self.buffers.push(Buffer::new(id, text, source));
        id
    }
    fn open_buffer(&mut self, id: usize) -> Result<&mut Buffer, EditorError> {
        let buf = self
            .buffers
            .iter_mut()
            .find(|b| b.id == id)
            .ok_or(EditorError::BufferNotFound(id))?;

        Ok(buf)
    }
    fn open_file(&mut self, filepath: &PathBuf) -> Result<usize, EditorError> {
        let id = self.max_id() + 1;
        self.buffers.push(Buffer::new_from_file(id, filepath)?);
        Ok(id)
    }
    fn get_current_window(&self) -> &Window {
        // hypothetically impossible to error, but oh well
        &self.current_window
    }

    // handles cursor movement within the window, scrolling, etc.
    // window movement within the actual TUI is handled by the render_frame function
    fn move_cursor(&mut self, direction: Direction) -> anyhow::Result<()> {
        let current_offset = self.current_window.scroll_offset;
        let current_line = self.current_window.cursor.line;
        let current_col = self.current_window.cursor.col;

        let (cols, rows) = terminal::size()?;

        if direction == Direction::Up && current_line - current_offset == 0 {
            // BUG: doesn't account for overflow
            self.current_window.scroll_offset -= 1;
        } else if direction == Direction::Down
            && ((current_line - current_offset) == (rows - 1).into())
        {
            self.current_window.scroll_offset += 1;
        }

        self.current_window.move_cursor(direction);

        Ok(())
    }
}

#[derive(Error, Debug)]
enum EditorError {
    #[error("buffer {0} not found")]
    BufferNotFound(usize),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
}

// TODO: rewrite this as a state machine that understands the window
fn render_frame(stdout: &mut impl Write, editor: &Editor) -> anyhow::Result<()> {
    let (cols, rows) = terminal::size()?;
    clear_screen(stdout)?;
    execute!(stdout, cursor::MoveTo(0, 0))?;

    let current_window = editor.get_current_window();

    // command buffer line in the bottom
    let num_lines = cols - 1;
    let offset = current_window.scroll_offset;
    let buffer = &editor.buffers[current_window.buffer_id];
    let buffer_num_lines = buffer.text.len();

    let start = offset;
    let end = (offset + num_lines as usize).min(buffer_num_lines);

    let lines = &buffer.text[start..end];

    let mut row = 0;
    for (line_number, line) in lines.iter().enumerate() {
        execute!(stdout, cursor::MoveTo(0, row))?;
        print!("{:>3} ", line_number + offset + 1);
        let mut col = 4;
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

fn main() -> anyhow::Result<()> {
    // let args = Args::parse();

    let mut stdout = std::io::stdout().lock();

    execute!(stdout, terminal::EnterAlternateScreen)?;

    enable_raw_mode()?;

    let mut editor = Editor::new();

    // main loop
    loop {
        render_frame(&mut stdout, &editor)?;
        match event::read()? {
            Event::Key(KeyEvent {
                code,
                modifiers,
                kind,
                state,
            }) => {
                if code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
                    println!("Exiting... (Ctrl+C)");
                    break;
                }
                match editor.mode {
                    Mode::Normal => match code {
                        KeyCode::Char('q') => {
                            break;
                        }
                        KeyCode::Char('h') => {
                            editor.move_cursor(Direction::Left)?;
                        }
                        KeyCode::Char('j') => {
                            editor.move_cursor(Direction::Down)?;
                        }
                        KeyCode::Char('k') => {
                            editor.move_cursor(Direction::Up)?;
                        }
                        KeyCode::Char('l') => {
                            editor.move_cursor(Direction::Right)?;
                        }
                        // KeyCode::Char('i') => {
                        //     mode = Mode::Insert;
                        // }
                        KeyCode::Char(':') => {
                            editor.mode = Mode::Command;
                        }
                        _ => {
                            // // for debug
                            // write!(stdout, "code:{} ", code)?;
                            // stdout.flush()?;
                        }
                    },
                    Mode::Insert => match code {
                        KeyCode::Char('i') => editor.mode = Mode::Insert,
                        KeyCode::Char('q') => editor.mode = Mode::Normal,
                        _ => {}
                    },
                    Mode::Command => match code {
                        KeyCode::Esc => {
                            editor.mode = Mode::Normal;
                        }
                        KeyCode::Enter => {
                            if editor.command_buffer == "q" {
                                break;
                            }
                            let command = editor.command_buffer.split_once(' ').unwrap().0;
                            if command == "e" {
                                let file = editor.command_buffer.split_once(' ').unwrap().1;
                                let id = editor.open_file(&PathBuf::from(file))?;
                                editor.current_window.buffer_id = id;
                            }
                            editor.command_buffer.clear();
                            editor.mode = Mode::Normal;
                        }
                        KeyCode::Backspace => {
                            if editor.command_buffer.is_empty() {
                                editor.mode = Mode::Normal;
                            } else {
                                editor.command_buffer.pop();
                            }
                        }
                        KeyCode::Char(c) => {
                            editor.command_buffer.push(c);
                        }
                        _ => {}
                    },
                }
            }
            _ => {}
        }
    }

    disable_raw_mode()?;
    println!("Bye!");
    Ok(())
}

fn clear_screen(stdout: &mut impl Write) -> std::io::Result<()> {
    execute!(stdout, Clear(crossterm::terminal::ClearType::All))?;
    Ok(())
}
