use std::io::Write;

use crate::{editor::Editor, window::WindowId};

#[derive(PartialEq, Debug)]
enum SplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug)]
enum Layout {
    Leaf(WindowId),
    Split {
        direction: SplitDirection,
        children: Vec<Layout>,
        split_at: u16,
    },
}

#[derive(Debug)]
pub struct Screen {
    tabs: Vec<Tab>,
}

#[derive(Debug)]
struct Tab {
    layout: Layout,
    focused_window: WindowId,
}

impl Screen {
    pub fn new() -> Self {
        Self {
            tabs: vec![Tab {
                layout: Layout::Leaf(WindowId(0)),
                focused_window: WindowId(0),
            }],
        }
    }
    pub fn render(&self, stdout: &mut impl Write, editor: &Editor) -> anyhow::Result<()> {
        Ok(())
    }
    pub fn set_command(&self, command: &str) {}
    pub fn paint(&self) {}
}
