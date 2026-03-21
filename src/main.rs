// use clap::Parser;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{self, Clear, disable_raw_mode, enable_raw_mode},
};
use std::io::Write;

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

fn main() -> anyhow::Result<()> {
    // let args = Args::parse();

    let mut stdout = std::io::stdout().lock();

    execute!(stdout, terminal::EnterAlternateScreen)?;

    enable_raw_mode()?;

    let mut mode = Mode::Normal;

    let mut command_buffer = String::with_capacity(1024);

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
