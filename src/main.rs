// use clap::Parser;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
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
    fn new_from_file(id: usize, filepath: &PathBuf) -> anyhow::Result<Self> {
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
struct Cursor {
    line: usize,
    col: usize,
}
impl Cursor {
    fn start() -> Self {
        Self { line: 0, col: 0 }
    }
}

struct Window {
    buffer_id: usize,
    cursor: Cursor,
    /// The absolute position of the first line that is visible
    scroll_offset: usize,
}

struct Editor {
    buffers: Vec<Buffer>,
    current_window: Window,
    command_buffer: String,
}

impl Editor {
    fn new() -> Self {
        let buf0 = Buffer::new_empty(0);
        Self {
            buffers: vec![buf0],
            current_window: Window {
                buffer_id: 0,
                cursor: Cursor::start(),
                scroll_offset: 0,
            },
            command_buffer: String::with_capacity(1024),
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
    fn get_current_buffer(&self) -> &Buffer {
        // hypothetically impossible to error, but oh well
        self.buffers.get(self.current_buffer).unwrap()
    }
}

#[derive(Error, Debug)]
enum EditorError {
    #[error("buffer {0} not found")]
    BufferNotFound(usize),
}

// 1. calculate // 1. render the current buffer
fn render_frame(stdout: &mut impl Write, editor: &Editor) -> anyhow::Result<()> {
    let (cols, rows) = terminal::size()?;
    clear_screen(stdout)?;

    for (line_number, line) in editor.get_current_buffer().text.iter().enumerate() {
        print!("{:>3} ", line_number + 1);
        print!("{}", line);
    }

    execute!(stdout, cursor::MoveTo(0, rows - 1))?;
    if !editor.command_buffer.is_empty() {
        print!(":{}", editor.command_buffer);
        stdout.flush()?;
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    // let args = Args::parse();

    let mut stdout = std::io::stdout().lock();

    execute!(stdout, terminal::EnterAlternateScreen)?;

    enable_raw_mode()?;

    let mut mode = Mode::Normal;

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
                match mode {
                    Mode::Normal => match code {
                        KeyCode::Char('q') => {
                            break;
                        }
                        // KeyCode::Char('i') => {
                        //     mode = Mode::Insert;
                        // }
                        KeyCode::Char(':') => {
                            mode = Mode::Command;
                        }
                        _ => {
                            // // for debug
                            // write!(stdout, "code:{} ", code)?;
                            // stdout.flush()?;
                        }
                    },
                    Mode::Insert => match code {
                        KeyCode::Char('i') => mode = Mode::Insert,
                        KeyCode::Char('q') => mode = Mode::Normal,
                        _ => {}
                    },
                    Mode::Command => {
                        let command_buffer = &mut editor.command_buffer;
                        match code {
                            KeyCode::Esc => {
                                mode = Mode::Normal;
                            }
                            KeyCode::Enter => {
                                if command_buffer == "q" {
                                    break;
                                }
                                command_buffer.clear();
                                mode = Mode::Normal;
                            }
                            KeyCode::Backspace => {
                                command_buffer.pop();
                            }
                            KeyCode::Char(c) => {
                                command_buffer.push(c);
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }

    disable_raw_mode()?;
    Ok(())
}

fn clear_screen(stdout: &mut impl Write) -> std::io::Result<()> {
    execute!(stdout, Clear(crossterm::terminal::ClearType::All))?;
    Ok(())
}
