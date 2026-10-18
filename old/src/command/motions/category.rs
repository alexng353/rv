#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Category {
    Word,
    Symbol,
    Whitespace,
}

pub fn categorize(c: char, big: bool) -> Category {
    if c.is_whitespace() {
        Category::Whitespace
    } else if big {
        Category::Word
    } else if c.is_alphanumeric() {
        Category::Word
    } else {
        Category::Symbol
    }
}
