use crate::{buffer::Buffer, command::motions::category::{Category, categorize}, window::BufferCursor};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum ForwardPhase {
    NoCategory,
    Start(Category),
    InWhitespace,
}

pub fn word_forwards(cursor: &BufferCursor, buffer: &Buffer, big: bool) -> BufferCursor {
    let mut phase = ForwardPhase::NoCategory;

    let mut line_idx = cursor.line;
    let mut col_idx = cursor.col;

    loop {
        if line_idx >= buffer.text.len() {
            let line = &buffer.text.len() - 1;
            return BufferCursor {
                col: buffer.text[line].chars().count() - 1,
                line,
            };
        }

        let line = &buffer.text[line_idx];
        let mut chars = line.chars().skip(col_idx);

        while let Some(c) = chars.next() {
            match phase {
                ForwardPhase::NoCategory => {
                    phase = ForwardPhase::Start(categorize(c, big));
                }
                ForwardPhase::Start(category) => {
                    if categorize(c, big) != category {
                        // we started with whitespace, we've now found a non-whitespace character
                        if category == Category::Whitespace {
                            break;
                        }
                        phase = ForwardPhase::InWhitespace;
                    }
                }
                ForwardPhase::InWhitespace => {
                    if categorize(c, big) != Category::Whitespace {
                        break;
                    }
                }
            }
            col_idx += 1;
        }

        let count = line.chars().count();

        if col_idx + 1 > count {
            col_idx = 0;
            line_idx += 1;
            phase = ForwardPhase::InWhitespace;
        } else {
            return BufferCursor {
                col: col_idx,
                line: line_idx,
            };
        }
    }
}

#[cfg(test)]
mod word_forward_tests {
    use super::*;
    use crate::buffer::Buffer;

    const FILE: &str = include_str!("../../../fixtures/lipsum.txt");

    #[test]
    fn normal_word() {
        let buffer = Buffer::new_from_str(0, FILE);
        let mut cursor = BufferCursor { line: 4, col: 0 };

        cursor = word_forwards(&cursor, &buffer, false);
        dbg!(&cursor);
        assert_eq!(cursor.line, 4);
        assert_eq!(cursor.col, 4);
    }

    #[test]
    fn word_at_end_of_line() {
        let buffer = Buffer::new_from_str(0, FILE);
        let mut cursor = BufferCursor { line: 4, col: 74 };

        cursor = word_forwards(&cursor, &buffer, false);
        dbg!(&cursor);
        assert_eq!(cursor.line, 5);
        assert_eq!(cursor.col, 0);
    }

    #[test]
    fn word_at_end_of_buffer() {
        let buffer = Buffer::new_from_str(0, FILE);
        let mut cursor = BufferCursor { line: 19, col: 0 };

        cursor = word_forwards(&cursor, &buffer, false);
        dbg!(&cursor);
        assert_eq!(cursor.line, 19);
        assert_eq!(cursor.col, 6);
    }

    #[test]
    fn edgecase_starting_whitespace() {
        let buffer = Buffer::new_from_str(0, "abc  def");
        let mut cursor = BufferCursor { line: 0, col: 4 };

        cursor = word_forwards(&cursor, &buffer, false);
        dbg!(&cursor);
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.col, 5);
    }
}
