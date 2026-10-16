use super::Position;

/// Selection: `anchor` stays, `head` moves (the caret). Collapsed = just a caret.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Selection {
    pub anchor: Position,
    pub head: Position,
}

impl Selection {
    pub fn caret(at: Position) -> Self {
        Self {
            anchor: at,
            head: at,
        }
    }

    pub fn is_collapsed(&self) -> bool {
        self.anchor == self.head
    }

    /// (start, end) in document order.
    pub fn range(&self) -> (Position, Position) {
        if self.anchor <= self.head {
            (self.anchor, self.head)
        } else {
            (self.head, self.anchor)
        }
    }
}
