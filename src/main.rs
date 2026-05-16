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
use tracing::{Level, info};

mod buffer;
mod command;
mod config;
mod editor;
mod errors;
mod keymap;
mod layout;
mod screen;
mod structs;
mod window;

use crate::{
    config::ConfigRaw,
    editor::{Editor, Mode},
    screen::Screen,
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
        .with_max_level(Level::DEBUG)
        .with_writer(non_blocking)
        .with_ansi(false) // disable color codes in file output
        .init();

    info!("Application started");
    let args = Args::parse();

    let mut frame_times: Vec<Duration> = vec![];

    let raw_config = ConfigRaw::get();
    std::fs::write("config.toml", &raw_config.debug_to_string()?)?;

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

        let mut editor = Editor::new(&raw_config)?;

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

            // TODO(P99): scripting language (rts)
            let event = event::read()?;

            if let Event::Key(key_event) = event {
                if let Ok(key) = key_event.try_into() {
                    editor.handle_key(key)?;
                }
            }

            if editor.should_quit {
                break;
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
