use std::io;

use crossterm::execute;
use crossterm::terminal::{self, disable_raw_mode, enable_raw_mode};

pub struct TerminalGuard;
impl TerminalGuard {
    pub fn new() -> io::Result<Self> {
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

pub fn set_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let _ = disable_raw_mode();
        let _ = execute!(std::io::stderr(), terminal::LeaveAlternateScreen);
        eprintln!("{}", info);
    }));
}
