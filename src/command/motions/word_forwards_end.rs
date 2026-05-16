use crate::{
    buffer::Buffer,
    command::motions::category::{Category, categorize},
    window::BufferCursor,
};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Phase {
    NoCategory,
    Start(Category),
}

fn advance_one(lines: &[String], mut line: usize, mut col: usize) -> (usize, usize) {
    col += 1;
    while line < lines.len() && col >= lines[line].chars().count() {
        line += 1;
        col = 0;
    }
    (line, col)
}

#[cfg(test)]
mod advance_one_tests {
    use super::advance_one;
    #[test]
    fn test_advance_one() {
        let lines = vec!["LINE ONE".to_string(), "LINE TWO".to_string()];

        let (line, col) = advance_one(&lines, 0, 0);
        assert_eq!(line, 0);
        assert_eq!(col, 1);
    }

    #[test]

    fn test_advance_one_wrap() {
        let lines = vec!["LINE ONE".to_string(), "LINE TWO".to_string()];

        let (line, col) = advance_one(&lines, 0, 7);
        assert_eq!(line, 1);
        assert_eq!(col, 0);
    }
}

fn past_end_of_buffer(lines: &[String], line: usize) -> bool {
    line >= lines.len()
}

fn buffer_end(lines: &[String]) -> (usize, usize) {
    if lines.is_empty() {
        return (0, 0);
    }
    let line = lines.len().saturating_sub(1);
    let col = lines[line].chars().count().saturating_sub(1);
    (line, col)
}

fn pure_word_forwards_end(lines: &[String], cursor: &BufferCursor, big: bool) -> (usize, usize) {
    let mut line = cursor.line;
    let mut col = cursor.col;

    (line, col) = advance_one(lines, line, col);
    if past_end_of_buffer(lines, line) {
        return buffer_end(lines);
    }

    let char_at =
        |line: usize, col: usize| -> char { lines[line].chars().nth(col).expect("out of bounds") };

    let past_eol = |line: usize, col: usize| -> bool {
        line > lines.len() || col > lines[line].chars().count()
    };

    let line_len = |line: usize| -> usize { lines[line].chars().count() };

    while past_eol(line, col) || categorize(char_at(line, col), big) == Category::Whitespace {
        (line, col) = advance_one(lines, line, col);
        if past_end_of_buffer(lines, line) {
            return buffer_end(lines);
        }
    }

    let starting_category = categorize(char_at(line, col), big);

    while col < line_len(line) && categorize(char_at(line, col), big) == starting_category {
        col += 1
    }

    col -= 1;

    (line, col)
}

pub fn word_forwards_end(cursor: &BufferCursor, buffer: &Buffer, big: bool) -> BufferCursor {
    let (line, col) = pure_word_forwards_end(&buffer.text, cursor, big);

    return BufferCursor { line, col };
}

#[cfg(test)]
mod word_forward_end_tests {
    use super::*;
    use crate::buffer::Buffer;

    const FILE: &str = include_str!("../../../fixtures/lipsum.txt");

    #[test]
    fn normal_word() {
        let buffer = Buffer::new_from_str(0, FILE);
        let mut cursor = BufferCursor { line: 0, col: 0 };

        cursor = word_forwards_end(&cursor, &buffer, false);
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.col, 3);
    }

    #[test]
    fn edgecase_end_of_word() {
        let buffer = Buffer::new_from_str(0, FILE);
        let mut cursor = BufferCursor { line: 0, col: 3 };

        cursor = word_forwards_end(&cursor, &buffer, false);
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.col, 8);
    }

    #[test]
    fn word_at_end_of_line() {
        let buffer = Buffer::new_from_str(0, FILE);
        let mut cursor = BufferCursor { line: 3, col: 10 };

        cursor = word_forwards_end(&cursor, &buffer, false);
        assert_eq!(cursor.line, 4);
        assert_eq!(cursor.col, 2);
    }

    #[test]
    fn word_at_end_of_buffer() {
        let buffer = Buffer::new_from_str(0, FILE);
        let mut cursor = BufferCursor { line: 19, col: 0 };

        cursor = word_forwards_end(&cursor, &buffer, false);
        assert_eq!(cursor.line, 19);
        assert_eq!(cursor.col, 5);
    }

    #[test]
    fn word_at_end_of_buffer_big() {
        let buffer = Buffer::new_from_str(0, FILE);
        let mut cursor = BufferCursor { line: 19, col: 0 };

        cursor = word_forwards_end(&cursor, &buffer, true);
        assert_eq!(cursor.line, 19);
        assert_eq!(cursor.col, 6);
    }

    #[test]
    fn edgecase_starting_whitespace() {
        let buffer = Buffer::new_from_str(0, "abc  def");
        let mut cursor = BufferCursor { line: 0, col: 3 };

        cursor = word_forwards_end(&cursor, &buffer, false);
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.col, 7);
    }
}
