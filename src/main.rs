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

struct Editor {
    buffers: Vec<Buffer>,
    current_buffer: usize,
}

impl Editor {
    fn new() -> Self {
        let buf0 = Buffer::new_empty(0);
        Self {
            buffers: vec![buf0],
            current_buffer: 0,
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
}

#[derive(Error, Debug)]
enum EditorError {
    #[error("buffer {0} not found")]
    BufferNotFound(usize),
}

fn main() -> anyhow::Result<()> {
    // let args = Args::parse();

    let mut stdout = std::io::stdout().lock();

    execute!(stdout, terminal::EnterAlternateScreen)?;

    enable_raw_mode()?;

    let mut mode = Mode::Normal;

    let mut command_buffer = String::with_capacity(1024);

    let mut editor = Editor::new();

    // main loop
    loop {
        let (cols, rows) = terminal::size()?;
        clear_screen(&mut stdout)?;

        execute!(stdout, cursor::MoveTo(0, rows - 1))?;
        if !command_buffer.is_empty() {
            print!(":{}", command_buffer);
            stdout.flush()?;
        }
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
                    Mode::Command => match code {
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
                    },
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
