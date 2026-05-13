use std::str::FromStr;

use anyhow::bail;

use crate::{editor::Mode, structs::Direction};

#[derive(Debug, Copy, Clone)]
pub enum Cycle {
    Next,
    Prev,
}

#[derive(Debug, Clone)]
pub enum Command {
    InsertChar(char),
    Insert(String),
    MoveCursor(Direction),
    CycleTab(Cycle),
    FocusWindow(Direction),
    CycleBuffer(Cycle),
    SetMode(Mode),
    Exit(u8),
}

impl FromStr for Command {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parsed = match s {
            "move_cursor_right" => Self::MoveCursor(Direction::Right),
            "move_cursor_left" => Self::MoveCursor(Direction::Left),
            "move_cursor_up" => Self::MoveCursor(Direction::Up),
            "move_cursor_down" => Self::MoveCursor(Direction::Down),

            "focus_window_right" => Self::FocusWindow(Direction::Right),
            "focus_window_left" => Self::FocusWindow(Direction::Left),
            "focus_window_up" => Self::FocusWindow(Direction::Up),
            "focus_window_down" => Self::FocusWindow(Direction::Down),

            "set_mode_insert" => Self::SetMode(Mode::Insert),
            "set_mode_normal" => Self::SetMode(Mode::Normal),
            "set_mode_command" => Self::SetMode(Mode::Command),

            "sigint" => Self::Exit(128 + 2),

            _ => bail!("Invalid config action: {}", s),
        };
        Ok(parsed)
    }
}
