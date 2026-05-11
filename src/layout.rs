use tracing::info;

use crate::{screen::SplitDirection, structs::Rect, window::WindowId};

#[derive(Debug, Clone)]
pub enum Layout {
    Leaf(WindowId),
    Split {
        direction: SplitDirection,
        split_at: u16,
        a: Box<Layout>,
        b: Box<Layout>,
    },
}

impl std::default::Default for Layout {
    fn default() -> Self {
        Self::Leaf(WindowId(0))
    }
}

impl Layout {
    pub fn walk(&self, parent: Rect) -> Vec<(WindowId, Rect)> {
        match self {
            Layout::Leaf(window_id) => vec![(*window_id, parent)],
            Layout::Split {
                direction,
                split_at,
                a,
                b,
            } => {
                let (a_rect, b_rect) = parent.split(*direction, *split_at);

                let a = a.walk(a_rect);
                let b = b.walk(b_rect);
                a.into_iter().chain(b).collect()
            }
        }
    }
    pub fn balance(&mut self, parent: Rect) -> &mut Self {
        match self {
            Layout::Leaf(_) => {}
            Layout::Split {
                direction,
                split_at,
                a,
                b,
            } => {
                let a_rect = Rect {
                    x: parent.x,
                    y: parent.y,
                    width: parent.width / 2,
                    height: parent.height,
                };
                let b_rect = Rect {
                    x: parent.x + parent.width / 2,
                    y: parent.y,
                    width: parent.width / 2,
                    height: parent.height,
                };
                let a = a.balance(a_rect);
                let b = b.balance(b_rect);
                *self = Layout::Split {
                    direction: *direction,
                    split_at: *split_at,
                    a: Box::new(a.clone()),
                    b: Box::new(b.clone()),
                };
            }
        }
        self
    }

    /// Split the layout btree at a target window.
    ///
    /// Returns true if that WindowId was found, or false if not.
    pub fn split_at(
        &mut self,
        target: WindowId,
        new_window: WindowId,
        direction: SplitDirection,
        rect: &Rect,
    ) -> bool {
        match self {
            Self::Leaf(id) => {
                if &target == id {
                    let split_at = match direction {
                        SplitDirection::Vertical => rect.width / 2,
                        SplitDirection::Horizontal => rect.height / 2,
                    };

                    *self = Layout::Split {
                        direction,
                        split_at,
                        a: Box::new(Layout::Leaf(*id)),
                        b: Box::new(Layout::Leaf(new_window)),
                    };

                    true
                } else {
                    false
                }
            }
            Self::Split {
                direction: dir,
                split_at: pos,
                a,
                b,
            } => {
                let (a_rect, b_rect) = rect.split(*dir, *pos);

                a.split_at(target, new_window, direction, &a_rect)
                    || b.split_at(target, new_window, direction, &b_rect)
            }
        }
    }
}
