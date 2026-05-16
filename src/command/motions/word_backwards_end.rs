use crate::{
    buffer::Buffer,
    command::motions::category::{Category, categorize},
    window::BufferCursor,
};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Phase {
    NoCategory,
    Start(Category),
    InWhitespace,
}

pub fn word_backwards_end(cursor: &BufferCursor, buffer: &Buffer, big: bool) -> BufferCursor {
    let mut phase = Phase::NoCategory;

    let line_idx = cursor.line;
    let mut col_idx = cursor.col;

    if col_idx == 0 {
        if line_idx == 0 {
            return BufferCursor { col: 0, line: 0 };
        }

        return BufferCursor {
            col: &buffer.text[line_idx - 1].chars().count() - 1,
            line: line_idx - 1,
        };
    }

    let line = &buffer.text[line_idx];
    let count = line.chars().count();
    let mut chars = line
        .chars()
        .rev()
        .skip(count.saturating_sub(1).saturating_sub(col_idx));

    while let Some(c) = chars.next() {
        dbg!(&col_idx, &c, &phase);
        let new = categorize(c, big);
        match phase {
            Phase::NoCategory => phase = Phase::Start(new),
            Phase::Start(category) => {
                if new != category {
                    if new == Category::Whitespace {
                        phase = Phase::InWhitespace;
                    } else {
                        break;
                    }
                }
            }
            Phase::InWhitespace => {
                if new != Category::Whitespace {
                    break;
                }
            }
        }
        col_idx -= 1;
    }

    return BufferCursor {
        col: col_idx,
        line: line_idx,
    };
}

#[cfg(test)]
mod word_backward_end_tests {
    use super::*;
    use crate::buffer::Buffer;

    const FILE: &str = include_str!("../../../fixtures/lipsum.txt");

    #[test]
    fn normal_word() {
        let buffer = Buffer::new_from_str(0, FILE);
        let mut cursor = BufferCursor { line: 4, col: 4 };

        cursor = word_backwards_end(&cursor, &buffer, false);
        assert_eq!(cursor.line, 4);
        assert_eq!(cursor.col, 2);
    }

    #[test]
    fn start_of_line() {
        let buffer = Buffer::new_from_str(0, FILE);
        let mut cursor = BufferCursor { line: 4, col: 0 };

        cursor = word_backwards_end(&cursor, &buffer, false);
        assert_eq!(cursor.line, 3);
        assert_eq!(cursor.col, 10);
    }

    #[test]
    fn end_of_line() {
        let buffer = Buffer::new_from_str(0, FILE);
        let mut cursor = BufferCursor { line: 3, col: 10 };

        cursor = word_backwards_end(&cursor, &buffer, false);
        assert_eq!(cursor.line, 3);
        assert_eq!(cursor.col, 9);
    }

    #[test]
    fn end_of_line_big() {
        let buffer = Buffer::new_from_str(0, FILE);
        let mut cursor = BufferCursor { line: 3, col: 10 };

        cursor = word_backwards_end(&cursor, &buffer, true);
        assert_eq!(cursor.line, 3);
        assert_eq!(cursor.col, 4);
    }
}
