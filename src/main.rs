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
    editor::{Editor, Mode},
    window::Direction,
};

/// A toy text editor in Rust
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The file to edit
    file: Option<String>,
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
        editor.current_window.buffer_id = id;
    }

    let mut frame_times: Vec<Duration> = vec![];

    // main loop
    loop {
        let start = Instant::now();
        render_frame(&mut stdout, &editor)?;
        let elapsed = start.elapsed();
        frame_times.push(elapsed);
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
                        KeyCode::Char(c) => editor.insert_char(c),
                        KeyCode::Esc => editor.mode = Mode::Normal,
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

    let mut total_time = Duration::ZERO;
    let len = frame_times.len() as u32;
    for frame_time in frame_times {
        total_time += frame_time;
    }
    info!("Average frame time: {:?}", total_time / len);
    Ok(())
}

fn clear_screen(stdout: &mut impl Write) -> std::io::Result<()> {
    execute!(stdout, Clear(crossterm::terminal::ClearType::All))?;
    Ok(())
}
