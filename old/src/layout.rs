use crate::{command::Axis, structs::Rect, window::WindowId};

#[derive(Debug, Clone)]
pub enum Layout {
    Leaf(WindowId),
    Split {
        axis: Axis,
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
                axis,
                split_at,
                a,
                b,
            } => {
                let (a_rect, b_rect) = parent.split(*axis, *split_at);

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
                axis,
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
                    axis: *axis,
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
        axis: Axis,
        rect: &Rect,
    ) -> bool {
        match self {
            Self::Leaf(id) => {
                if &target == id {
                    let split_at = match axis {
                        Axis::Vertical => rect.width / 2,
                        Axis::Horizontal => rect.height / 2,
                    };

                    *self = Layout::Split {
                        axis,
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
                axis: dir,
                split_at: pos,
                a,
                b,
            } => {
                let (a_rect, b_rect) = rect.split(*dir, *pos);

                a.split_at(target, new_window, axis, &a_rect)
                    || b.split_at(target, new_window, axis, &b_rect)
            }
        }
    }
}
