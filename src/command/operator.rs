use crate::{
    buffer::Buffer,
    command::range::{Range, RangeKind},
    register::Register,
    window::BufferCursor,
};
use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Delete,
    Change,
    Yank,
}

pub struct OperatorOutcome {
    pub enter_insert: bool,
}

fn extract_chars(buffer: &Buffer, range: Range) -> String {
    let mut chars = String::new();

    match range.kind {
        RangeKind::Linewise => {
            chars.push_str(&buffer.text[range.from.line..=range.to.line].join("\n"));
        }
        RangeKind::Charwise => {
            let lines = &buffer.text[range.from.line..=range.to.line];

            // charwise
            let last = lines.len().saturating_sub(1);
            for (i, line) in lines.iter().enumerate() {
                let is_first = i == 0;
                let is_last = i == last;

                if is_first && is_last {
                    if range.inclusive {
                        chars.push_str(&line[range.from.col..=range.to.col]);
                    } else {
                        chars.push_str(&line[range.from.col..range.to.col]);
                    }
                } else if is_first {
                    chars.push_str(&line[range.from.col..]);
                } else if is_last {
                    if range.inclusive {
                        chars.push_str(&line[..=range.to.col]);
                    } else {
                        chars.push_str(&line[..range.to.col]);
                    }
                } else {
                    chars.push_str(line);
                }
            }
        }
        RangeKind::Blockwise => todo!(),
    }

    chars
}

pub fn apply_operator(
    op: Operator,
    range: Range,
    buffer: &mut Buffer,
    register: &mut Register,
) -> OperatorOutcome {
    let mut enter_insert = false;

    match op {
        Operator::Delete => {
            register.set(extract_chars(buffer, range), range.kind);
            match range.kind {
                RangeKind::Linewise => {
                    buffer.delete_lines(range.from.line, range.to.line);
                }
                RangeKind::Charwise => {
                    buffer.delete_range(range.from, range.to, range.inclusive);
                }
                RangeKind::Blockwise => {
                    buffer.delete_range(range.from, range.to, range.inclusive);
                }
            }
        }
        Operator::Change => {
            apply_operator(Operator::Delete, range, buffer, register);
            enter_insert = true;
        }
        Operator::Yank => {
            register.set(extract_chars(buffer, range), range.kind);
        }
    }

    OperatorOutcome { enter_insert }
}
