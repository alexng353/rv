use crate::{
    buffer::Buffer,
    command::motions::{word_backwards, word_backwards_end, word_forwards, word_forwards_end},
    editor::Mode,
    window::BufferCursor,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Copy, Clone)]
pub enum Placement {
    High,
    Middle,
    Low,
}

#[derive(Debug, Copy, Clone)]
pub enum Motion {
    /// h, j, k, l
    Cursor(Direction),
    /// w, b, e, W, B, E
    Word { forward: bool, big: bool, end: bool },
    /// 0
    LineZero,
    /// ^
    LineStart,
    /// $
    LineEnd,
    /// gg
    FileTop,
    /// G
    FileBottom,
}

impl Motion {
    pub fn target(&self, cursor: &BufferCursor, buffer: &Buffer, mode: &Mode) -> BufferCursor {
        let mut out = match self {
            Motion::Cursor(direction) => {
                let (col, line) = match direction {
                    Direction::Up => (cursor.col, cursor.line.saturating_sub(1)),
                    Direction::Down => (cursor.col, cursor.line + 1),
                    Direction::Left => (cursor.col.saturating_sub(1), cursor.line),
                    Direction::Right => (cursor.col + 1, cursor.line),
                };
                BufferCursor { col, line }
            }
            Motion::Word { forward, big, end } => {
                match (forward, end) {
                    (true, false) => word_forwards(&cursor, buffer, *big),
                    (false, false) => word_backwards(&cursor, buffer, *big),
                    (true, true) => word_forwards_end(&cursor, buffer, *big),
                    (false, true) => word_backwards_end(&cursor, buffer, *big),
                }
            }
            Motion::LineZero => BufferCursor {
                col: 0,
                line: cursor.line,
            },
            Motion::LineStart => {
                let line = &buffer.text[cursor.line];

                BufferCursor {
                    // first non-whitespace character, or 0
                    col: line.find(|c: char| !c.is_whitespace()).unwrap_or(0),
                    line: cursor.line,
                }
            }
            Motion::LineEnd => {
                let line_length = buffer.text[cursor.line].len();
                BufferCursor {
                    col: line_length,
                    line: cursor.line,
                }
            }
            Motion::FileTop => BufferCursor {
                col: cursor.col,
                line: 0,
            },
            Motion::FileBottom => BufferCursor {
                col: cursor.col,
                line: buffer.text.len() - 1,
            },
        };

        out = out.cursor_clamp(buffer, mode);

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::Buffer;

    const FILE: &str = include_str!("../../fixtures/lipsum.txt");

    #[test]
    fn normal_motion_directions() {
        let buffer = Buffer::new_from_str(0, FILE);
        let mut cursor = BufferCursor::start();
        let mode = Mode::Normal;

        cursor = Motion::Cursor(Direction::Down).target(&cursor, &buffer, &mode);
        assert_eq!(cursor.line, 1);

        cursor = Motion::Cursor(Direction::Up).target(&cursor, &buffer, &mode);
        assert_eq!(cursor.line, 0);

        cursor = Motion::Cursor(Direction::Right).target(&cursor, &buffer, &mode);
        assert_eq!(cursor.col, 1);

        cursor = Motion::Cursor(Direction::Left).target(&cursor, &buffer, &mode);
        assert_eq!(cursor.col, 0);
    }

    #[test]
    fn line_positions() {
        let buffer = Buffer::new_from_str(0, FILE);
        let mut cursor = BufferCursor::start();
        let mode = Mode::Normal;

        cursor = Motion::LineEnd.target(&cursor, &buffer, &mode);
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.col, 247);

        cursor = Motion::LineZero.target(&cursor, &buffer, &mode);
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.col, 0);

        cursor = Motion::Cursor(Direction::Down).target(&cursor, &buffer, &mode);
        assert_eq!(cursor.line, 1);
        assert_eq!(cursor.col, 0);

        cursor = Motion::LineStart.target(&cursor, &buffer, &mode);
        assert_eq!(cursor.line, 1);
        assert_eq!(cursor.col, 12);
    }

    #[test]
    fn file_positions() {
        let buffer = Buffer::new_from_str(0, FILE);
        let mut cursor = BufferCursor::start();
        let mode = Mode::Normal;

        cursor = Motion::FileBottom.target(&cursor, &buffer, &mode);
        assert_eq!(cursor.line, 19);
        assert_eq!(cursor.col, 0);

        cursor = Motion::FileTop.target(&cursor, &buffer, &mode);
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.col, 0);
    }

    #[test]
    fn mode_clamp() {
        let buffer = Buffer::new_from_str(0, FILE);
        let mut cursor = BufferCursor { line: 3, col: 0 };
        let mut mode = Mode::Normal;

        cursor = Motion::LineEnd.target(&cursor, &buffer, &mode);
        assert_eq!(cursor.col, 10);

        mode = Mode::Insert;
        cursor = Motion::LineEnd.target(&cursor, &buffer, &mode);
        assert_eq!(cursor.col, 11);

        mode = Mode::Normal;
        cursor = cursor.cursor_clamp(&buffer, &mode);
        assert_eq!(cursor.col, 10);
    }

    #[test]
    fn word() {
        let buffer = Buffer::new_from_str(0, FILE);
        let mut cursor = BufferCursor { line: 4, col: 0 };

        cursor = Motion::Word {
            forward: true,
            big: false,
            end: false,
        }
        .target(&cursor, &buffer, &Mode::Normal);
        assert_eq!(cursor.line, 4);
        assert_eq!(cursor.col, 4);

        cursor = Motion::LineEnd.target(&cursor, &buffer, &Mode::Normal);
        cursor = Motion::Word {
            forward: true,
            big: false,
            end: false,
        }
        .target(&cursor, &buffer, &Mode::Normal);

        assert_eq!(cursor.line, 5);
        assert_eq!(cursor.col, 0);

        cursor = BufferCursor { line: 6, col: 34 };
        cursor = Motion::Word {
            forward: true,
            big: false,
            end: false,
        }
        .target(&cursor, &buffer, &Mode::Normal);

        assert_eq!(cursor.line, 7);
        assert_eq!(cursor.col, 0);
    }
}
