use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{self, disable_raw_mode, enable_raw_mode},
};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use tracing::info;

mod buffer;
mod editor;
mod errors;
mod layout;
mod screen;
mod structs;
mod window;

use crate::{
    editor::{Editing, Editor, Mode},
    screen::{Screen, SplitDirection},
    window::Direction,
};

/// A toy text editor in Rust
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The file to edit
    file: Option<String>,
}

struct TerminalGuard;
impl TerminalGuard {
    fn new() -> anyhow::Result<Self> {
        enable_raw_mode()?;
        execute!(std::io::stdout(), terminal::EnterAlternateScreen)?;
        Ok(Self)
    }
}
impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(std::io::stdout(), terminal::LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
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

    let mut frame_times: Vec<Duration> = vec![];

    {
        std::panic::set_hook(Box::new(|info| {
            let _ = disable_raw_mode();
            let _ = execute!(std::io::stderr(), terminal::LeaveAlternateScreen);
            eprintln!("{}", info);
        }));
        // must come before stdout is locked
        // RAII cleanup on drop for stdout
        let _guard = TerminalGuard::new()?;
        let mut stdout = std::io::stdout().lock();

        let mut editor = Editor::new();

        if let Some(file) = args.file {
            let id = editor.open_file(&PathBuf::from(file))?;
            editor.windows[0].buffer_id = id;
        }

        let mut screen = Screen::new();

        // main loop
        loop {
            let start = Instant::now();
            screen.render(&mut stdout, &editor)?;
            let elapsed = start.elapsed();
            frame_times.push(elapsed);
            let current_window_id = screen.current_window_id();
            // TODO: move keybinds to another file/method
            // TODO(P2): configuration for bindings (TOML, perhaps)
            // TODO(P99): scripting language (rts)
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
                                info!("command_buffer: {}", editor.command_buffer);
                                match editor.command_buffer.as_str() {
                                    "q" => {
                                        break;
                                    }
                                    "split" => {
                                        let scratch =
                                            editor.new_buffer(vec![], buffer::BufSource::Scratch);
                                        let window = editor.new_window(scratch);

                                        screen.split(window, SplitDirection::Horizontal)?;
                                    }
                                    "vsplit" => {
                                        let scratch =
                                            editor.new_buffer(vec![], buffer::BufSource::Scratch);
                                        let window = editor.new_window(scratch);

                                        screen.split(window, SplitDirection::Vertical)?;
                                    }
                                    _ => {}
                                };

                                if let Some(command) = editor.command_buffer.split_once(' ') {
                                    if command.0 == "e" {
                                        let id = editor.open_file(&PathBuf::from(command.1))?;
                                        // TODO: this is a hack
                                        editor.windows[current_window_id].buffer_id = id;
                                    }
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
    }
    println!("Bye!");

    let len = frame_times.len() as u32;
    let total_time = frame_times.iter().sum::<Duration>();

    info!("Average frame time: {:?}", total_time / len);
    println!("Average frame time: {:?}", total_time / len);
    Ok(())
}
