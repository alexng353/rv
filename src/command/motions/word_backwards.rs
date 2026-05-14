use crate::{buffer::Buffer, command::motions::category::{Category, categorize}, window::BufferCursor};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum BackwardPhase {
    NoCategory,
    Start(Category),
}

pub fn word_backwards(cursor: &BufferCursor, buffer: &Buffer, big: bool) -> BufferCursor {
    let mut phase = BackwardPhase::NoCategory;

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
    let mut chars = line.chars().rev().skip(count - col_idx);

    while let Some(c) = chars.next() {
        match phase {
            BackwardPhase::NoCategory => {
                phase = BackwardPhase::Start(categorize(c, big));
            }
            BackwardPhase::Start(category) => {
                // detect transition
                if categorize(c, big) != category {
                    // we started with whitespace, we've now found a non-whitespace character
                    if category == Category::Whitespace {
                        phase = BackwardPhase::Start(categorize(c, big));
                    } else {
                        return BufferCursor {
                            col: col_idx,
                            line: line_idx,
                        };
                    }
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
mod word_backward_tests {
    use super::*;
    use crate::buffer::Buffer;

    const FILE: &str = include_str!("../../../fixtures/lipsum.txt");

    #[test]
    fn normal_word() {
        let buffer = Buffer::new_from_str(0, FILE);
        let mut cursor = BufferCursor { line: 4, col: 4 };

        cursor = word_backwards(&cursor, &buffer, false);
        assert_eq!(cursor.line, 4);
        assert_eq!(cursor.col, 0);
    }

    #[test]
    fn start_of_line() {
        let buffer = Buffer::new_from_str(0, FILE);
        let mut cursor = BufferCursor { line: 4, col: 0 };

        cursor = word_backwards(&cursor, &buffer, false);
        assert_eq!(cursor.line, 3);
        assert_eq!(cursor.col, 10);
    }

    #[test]
    fn end_of_line() {
        let buffer = Buffer::new_from_str(0, FILE);
        let mut cursor = BufferCursor { line: 3, col: 10 };

        cursor = word_backwards(&cursor, &buffer, false);
        assert_eq!(cursor.line, 3);
        assert_eq!(cursor.col, 6);
    }
}
