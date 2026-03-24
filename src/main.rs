use clap::Parser;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{self, Clear, disable_raw_mode, enable_raw_mode},
};
use std::{
    io::Write,
    path::PathBuf,
    time::{Duration, Instant},
};
use tracing::{error, info, warn};

mod buffer;
mod editor;
mod errors;
mod screen;
mod window;

use crate::{
    editor::{Editing, Editor, Mode},
    screen::Screen,
    window::Direction,
};

/// A toy text editor in Rust
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The file to edit
    file: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let file_appender = tracing_appender::rolling::never("logs", "log.txt");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false) // disable color codes in file output
        .init();

    info!("Application started");
    let args = Args::parse();

    let mut stdout = std::io::stdout().lock();

    execute!(stdout, terminal::EnterAlternateScreen)?;

    std::panic::set_hook(Box::new(|info| {
        let _ = disable_raw_mode();
        let _ = execute!(std::io::stderr(), terminal::LeaveAlternateScreen);
        eprintln!("{}", info);
    }));

    enable_raw_mode()?;

    let mut editor = Editor::new();

    if let Some(file) = args.file {
        let id = editor.open_file(&PathBuf::from(file))?;
        editor.windows[0].buffer_id = id;
    }

    let mut screen = Screen::new();

    let mut frame_times: Vec<Duration> = vec![];

    // main loop
    loop {
        let start = Instant::now();
        screen.render(&mut stdout, &editor)?;
        let elapsed = start.elapsed();
        frame_times.push(elapsed);
        let current_window_id = screen.current_window_id();
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
                            editor.move_cursor(current_window_id, Direction::Left)?;
                        }
                        KeyCode::Char('j') => {
                            editor.move_cursor(current_window_id, Direction::Down)?;
                        }
                        KeyCode::Char('k') => {
                            editor.move_cursor(current_window_id, Direction::Up)?;
                        }
                        KeyCode::Char('l') => {
                            editor.move_cursor(current_window_id, Direction::Right)?;
                        }
                        KeyCode::Char('i') => {
                            editor.mode = Mode::Insert;
                        }
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
                        KeyCode::Char(c) => editor.insert_char(current_window_id, c),
                        KeyCode::Esc => editor.mode = Mode::Normal,
                        KeyCode::Backspace => editor.backspace(current_window_id),
                        KeyCode::Enter => editor.enter(current_window_id),
                        // KeyCode::Char('i') => editor.mode = Mode::Insert,
                        // KeyCode::Char('q') => editor.mode = Mode::Normal,
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
                            // TODO: there's no guarantee that this is going to be a valid command
                            let command = editor.command_buffer.split_once(' ').unwrap().0;
                            if command == "e" {
                                let file = editor.command_buffer.split_once(' ').unwrap().1;
                                let id = editor.open_file(&PathBuf::from(file))?;
                                // TODO: this is a hack
                                editor.windows[current_window_id].buffer_id = id;
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

    let mut total_time = Duration::ZERO;
    let len = frame_times.len() as u32;
    for frame_time in frame_times {
        total_time += frame_time;
    }
    info!("Average frame time: {:?}", total_time / len);
    Ok(())
}
