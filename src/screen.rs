use crate::{editor::Editor, window::WindowId};

#[derive(PartialEq)]
enum SplitDirection {
    Horizontal,
    Vertical,
}

enum Layout {
    Leaf(WindowId),
    Split {
        direction: SplitDirection,
        children: Vec<Layout>,
        split_at: u16,
    },
}

pub struct Screen<'a> {
    command: Option<&'a str>,
    tabs: Vec<Tab>,
}

struct Tab {
    layout: Layout,
    focused_window: WindowId,
}

impl Screen<'_> {
    pub fn new() -> Self {
        Self {
            command: None,
            tabs: vec![Tab {
                layout: Layout::Leaf(WindowId(0)),
                focused_window: WindowId(0),
            }],
        }
    }
    pub fn render(&self, editor: &Editor) {}
    pub fn set_command(&self, command: &str) {}
    pub fn paint(&self) {}
}
