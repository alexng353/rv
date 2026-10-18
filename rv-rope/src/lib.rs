use std::{
    fmt::Write,
    ops::{Add, Sub},
    str::FromStr,
    sync::Arc,
};

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
struct ByteOffset(usize);
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
struct CharOffset(usize);
impl Add for CharOffset {
    type Output = CharOffset;

    fn add(self, rhs: Self) -> Self::Output {
        CharOffset(self.0 + rhs.0)
    }
}

impl Sub for CharOffset {
    type Output = CharOffset;

    fn sub(self, rhs: Self) -> Self::Output {
        CharOffset(self.0 - rhs.0)
    }
}
struct LineIdx(usize);

#[derive(Debug, Eq, PartialEq)]
struct Rope {
    root: Arc<Node>,
}

const MAX_BYTES: usize = 1024;

impl Rope {
    pub fn new() -> Self {
        Self {
            root: Arc::new(Node::new()),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_empty()
    }

    pub fn insert(&mut self, i: CharOffset, s: &str) {
        let (l, r) = split_nodes(self.root.clone(), i);
        self.root = cat_nodes(cat_nodes(l, Node::from_str(s).into()), r);
    }

    pub fn delete(&mut self, start: CharOffset, end: CharOffset) {
        let (l, r) = split_nodes(self.root.clone(), start);
        let (_, end) = split_nodes(r, end);
        self.root = cat_nodes(l, end);
    }
}

impl std::fmt::Display for Rope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.root.write_into(f)
    }
}

#[derive(Debug, Eq, PartialEq)]
enum Node {
    Leaf {
        content: String,
        bytes: usize,
        chars: CharOffset,
        newlines: usize,
    },
    Internal {
        left: Arc<Node>,
        right: Arc<Node>,
        bytes: usize,
        chars: CharOffset,
        newlines: usize,
    },
}

impl Node {
    pub fn new() -> Self {
        Self::Leaf {
            content: String::new(),
            bytes: 0,
            chars: CharOffset(0),
            newlines: 0,
        }
    }

    fn is_empty(&self) -> bool {
        matches!(self, Node::Leaf{content, ..} if content.is_empty())
    }

    fn write_into(&self, out: &mut impl std::fmt::Write) -> std::fmt::Result {
        match self {
            Self::Leaf { content, .. } => out.write_str(&content),
            Self::Internal { left, right, .. } => {
                left.write_into(out)?;
                right.write_into(out)?;
                Ok(())
            }
        }
    }

    fn bytes(&self) -> usize {
        match self {
            Node::Leaf { bytes, .. } => bytes.to_owned(),
            Node::Internal { bytes, .. } => bytes.to_owned(),
        }
    }

    fn chars(&self) -> CharOffset {
        match self {
            Node::Leaf { chars, .. } => chars.to_owned(),
            Node::Internal { chars, .. } => chars.to_owned(),
        }
    }

    fn newlines(&self) -> usize {
        match self {
            Node::Leaf { newlines, .. } => newlines.to_owned(),
            Node::Internal { newlines, .. } => newlines.to_owned(),
        }
    }

    fn from_str(s: &str) -> Self {
        Node::Leaf {
            content: s.to_string(),
            bytes: s.len(),
            chars: CharOffset(s.chars().count()),
            newlines: s.matches('\n').count(),
        }
    }
}

fn cat(a: Rope, b: Rope) -> Rope {
    if a.is_empty() {
        return b;
    } else if b.is_empty() {
        return a;
    }

    let a = a.root;
    let b = b.root;

    Rope {
        root: Arc::new(Node::Internal {
            bytes: a.bytes() + b.bytes(),
            chars: a.chars() + b.chars(),
            newlines: a.newlines() + b.newlines(),
            left: a,
            right: b,
        }),
    }
}

fn cat_nodes(a: Arc<Node>, b: Arc<Node>) -> Arc<Node> {
    if a.is_empty() {
        return b;
    } else if b.is_empty() {
        return a;
    }

    Arc::new(Node::Internal {
        bytes: a.bytes() + b.bytes(),
        chars: a.chars() + b.chars(),
        newlines: a.newlines() + b.newlines(),
        left: a,
        right: b,
    })
}

fn split(r: Rope, i: CharOffset) -> (Rope, Rope) {
    let (a, b) = split_nodes(r.root, i);
    (Rope { root: a }, Rope { root: b })
}

fn split_nodes(a: Arc<Node>, i: CharOffset) -> (Arc<Node>, Arc<Node>) {
    if i.0 == 0 {
        // split at start, return (empty, a)
        return (Node::new().into(), a);
    } else if i == a.chars() {
        // split at end, return (a, empty)
        return (a, Node::new().into());
    }

    match &*a {
        Node::Leaf {
            content,
            bytes,
            chars,
            newlines,
        } => {
            debug_assert!(i <= *chars);

            let char_idx = content
                .char_indices()
                .nth(i.0)
                .map(|f| f.0)
                .unwrap_or(content.len());

            let left = &content[..char_idx];
            let right = &content[char_idx..];

            return (
                Arc::new(Node::from_str(left)),
                Arc::new(Node::from_str(right)),
            );
        }
        Node::Internal { left, right, .. } => {
            let lw = left.chars();
            if i < lw {
                let (ll, lr) = split_nodes(left.clone(), i);
                return (ll, cat_nodes(lr, right.clone()));
            } else if i > lw {
                let (rl, rr) = split_nodes(right.clone(), i - lw);
                return (cat_nodes(left.clone(), rl), rr);
            } else {
                return (left.clone(), right.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rope_from_str(str: &str) -> Rope {
        Rope {
            root: Arc::new(Node::Leaf {
                content: str.to_string(),
                bytes: str.len(),
                chars: CharOffset(str.chars().count()),
                newlines: str.matches('\n').count(),
            }),
        }
    }

    #[test]
    fn cat_simple() {
        let hello = rope_from_str("Hello, ");
        let world = rope_from_str("world!");

        let hello_world = cat(hello, world);

        assert_eq!(hello_world.root.bytes(), 13);
        assert_eq!(hello_world.root.chars().0, 13);
        assert_eq!(hello_world.root.newlines(), 0);

        assert_eq!(hello_world.to_string(), "Hello, world!");
    }

    #[test]
    fn split_node_simple() {
        let hello_world = Node::from_str("Hello, world!");
        let (a, b) = split_nodes(Arc::new(hello_world), CharOffset(7));

        let hello = Arc::new(Node::from_str("Hello, "));
        let world = Arc::new(Node::from_str("world!"));

        assert_eq!(a, hello);
        assert_eq!(b, world);
    }

    #[test]
    fn split_simple() {
        let hello_world = rope_from_str("Hello, world!");
        let (a, b) = split(hello_world, CharOffset(7));

        let hello = rope_from_str("Hello, ");
        let world = rope_from_str("world!");

        assert_eq!(a, hello);
        assert_eq!(b, world);
    }

    #[test]
    fn split_node_nonascii() {
        let konnichiwa = Arc::new(Node::from_str("こんにちは"));
        let (a, b) = split_nodes(konnichiwa, CharOffset(2));

        let kon = Arc::new(Node::from_str("こん"));
        let nichiwa = Arc::new(Node::from_str("にちは"));

        assert_eq!(a, kon);
        assert_eq!(b, nichiwa);
    }

    #[test]
    fn rope_insert_ascii() {
        let mut rope = Rope::new();

        rope.insert(CharOffset(0), "Hello, ");
        assert_eq!(rope.to_string(), "Hello, ");

        rope.insert(CharOffset(7), "world!");
        assert_eq!(rope.to_string(), "Hello, world!");

        rope.insert(CharOffset(7), "sigma ");
        assert_eq!(rope.to_string(), "Hello, sigma world!");
    }

    #[test]
    fn rope_insert_nonascii() {
        let mut rope = Rope::new();

        rope.insert(CharOffset(0), "こん");
        assert_eq!(rope.to_string(), "こん");

        rope.insert(CharOffset(2), "ちは");
        assert_eq!(rope.to_string(), "こんちは");

        rope.insert(CharOffset(2), "に");
        assert_eq!(rope.to_string(), "こんにちは")
    }
}
