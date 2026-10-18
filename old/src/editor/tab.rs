use crate::{layout::Layout, structs::Rect, window::WindowId};

#[derive(Debug)]
pub struct Tab {
    pub(crate) layout: Layout,
    pub(crate) focused_window: WindowId,
}

impl Tab {
    pub fn walk(&self, rect: Rect) -> Vec<(WindowId, Rect)> {
        self.layout.walk(rect)
    }

    pub fn set_window(&mut self, window_id: WindowId) {
        self.focused_window = window_id;
    }
}
