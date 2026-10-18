use crate::window::BufferCursor;

#[derive(Debug, Clone, Copy)]
pub enum RangeKind {
    Charwise,
    Linewise,
    Blockwise,
}

#[derive(Debug, Clone, Copy)]
pub struct Range {
    pub from: BufferCursor,
    pub to: BufferCursor,
    pub kind: RangeKind,
    pub inclusive: bool,
}

pub fn normalize(range: Range) -> Range {
    if range.from > range.to {
        Range {
            from: range.to,
            to: range.from,
            kind: range.kind,
            inclusive: range.inclusive,
        }
    } else {
        range
    }
}
