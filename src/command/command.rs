use std::str::FromStr;

use anyhow::bail;

use crate::{
    command::{
        Motion,
        motion::{Direction, Placement},
    },
    editor::Mode,
};

#[derive(Debug, Copy, Clone)]
pub enum Cycle {
    Next,
    Prev,
}

#[derive(Debug, Copy, Clone)]
pub enum Scroll {
    CursorRelative(Placement),
    LineWise(Direction),
    Page { up: bool, full: bool },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone)]
pub enum Command {
    InsertChar(char),
    Insert(String),
    Move(Motion),
    Scroll(Scroll),
    CycleTab(Cycle),
    FocusWindow(Direction),
    CycleBuffer(Cycle),
    SetMode(Mode),
    Exit(u8),
    Append,
    AppendEol,
    InsertLineStart,
    InsertZero,
    Split(Axis),
}

impl FromStr for Command {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parsed = match s {
            "move_cursor_right" => Self::Move(Motion::Cursor(Direction::Right)),
            "move_cursor_left" => Self::Move(Motion::Cursor(Direction::Left)),
            "move_cursor_up" => Self::Move(Motion::Cursor(Direction::Up)),
            "move_cursor_down" => Self::Move(Motion::Cursor(Direction::Down)),

            "move_line_zero" => Self::Move(Motion::LineZero),
            "move_line_start" => Self::Move(Motion::LineStart),
            "move_line_end" => Self::Move(Motion::LineEnd),

            "move_file_top" => Self::Move(Motion::FileTop),
            "move_file_bottom" => Self::Move(Motion::FileBottom),

            "scroll_up" => Self::Scroll(Scroll::LineWise(Direction::Up)),
            "scroll_down" => Self::Scroll(Scroll::LineWise(Direction::Down)),
            "scroll_left" => Self::Scroll(Scroll::CursorRelative(Placement::Low)),
            "scroll_right" => Self::Scroll(Scroll::CursorRelative(Placement::High)),
            "scroll_page_up" => Self::Scroll(Scroll::Page { up: true, full: false }),
            "scroll_page_down" => Self::Scroll(Scroll::Page { up: false, full: false }),
            "scroll_page_up_full" => Self::Scroll(Scroll::Page { up: true, full: true }),
            "scroll_page_down_full" => Self::Scroll(Scroll::Page { up: false, full: true }),

            "cycle_tab_next" => Self::CycleTab(Cycle::Next),
            "cycle_tab_prev" => Self::CycleTab(Cycle::Prev),

            "jump_word_forward" => Self::Move(Motion::Word {
                forward: true,
                big: false,
                end: false,
            }),
            "jump_word_backward" => Self::Move(Motion::Word {
                forward: false,
                big: false,
                end: false,
            }),
            "jump_word_big_forward" => Self::Move(Motion::Word {
                forward: true,
                big: true,
                end: false,
            }),
            "jump_word_big_backward" => Self::Move(Motion::Word {
                forward: false,
                big: true,
                end: false,
            }),
            "jump_word_end_forward" => Self::Move(Motion::Word {
                forward: true,
                big: false,
                end: true,
            }),
            "jump_word_end_backward" => Self::Move(Motion::Word {
                forward: false,
                big: false,
                end: true,
            }),
            "jump_word_big_end_forward" => Self::Move(Motion::Word {
                forward: true,
                big: true,
                end: true,
            }),
            "jump_word_big_end_backward" => Self::Move(Motion::Word {
                forward: false,
                big: true,
                end: true,
            }),

            "focus_window_right" => Self::FocusWindow(Direction::Right),
            "focus_window_left" => Self::FocusWindow(Direction::Left),
            "focus_window_up" => Self::FocusWindow(Direction::Up),
            "focus_window_down" => Self::FocusWindow(Direction::Down),

            "split_horizontal" => Self::Split(Axis::Horizontal),
            "split_vertical" => Self::Split(Axis::Vertical),

            "set_mode_insert" => Self::SetMode(Mode::Insert),
            "set_mode_normal" => Self::SetMode(Mode::Normal),
            "set_mode_command" => Self::SetMode(Mode::Command),

            "append" => Self::Append,
            "append_eol" => Self::AppendEol,
            "insert_line_start" => Self::InsertLineStart,
            "insert_zero" => Self::InsertZero,

            "sigint" => Self::Exit(128 + 2),

            _ => bail!("Invalid config action: {}", s),
        };
        Ok(parsed)
    }
}

