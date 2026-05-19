use crate::command::range::RangeKind;

#[derive(Debug, Clone)]
pub struct Register {
    content: String,
    kind: RangeKind,
}

impl Register {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            kind: RangeKind::Charwise,
        }
    }

    pub fn set(&mut self, content: String, kind: RangeKind) {
        self.content = content;
        self.kind = kind;
    }
}
