use super::Edit;

/// Undo/redo stacks.
#[derive(Default)]
pub struct History {
    undo: Vec<Edit>,
    redo: Vec<Edit>,
}

impl History {
    /// Records a new edit; this clears the redo stack.
    pub fn push(&mut self, edit: Edit) {
        self.undo.push(edit);
        self.redo.clear();
    }

    /// The latest edit, to fold a continuing gesture (a drag) into one undo step.
    pub fn last_mut(&mut self) -> Option<&mut Edit> {
        self.undo.last_mut()
    }

    pub fn pop_undo(&mut self) -> Option<Edit> {
        let edit = self.undo.pop()?;
        self.redo.push(edit.clone());
        Some(edit)
    }

    pub fn pop_redo(&mut self) -> Option<Edit> {
        let edit = self.redo.pop()?;
        self.undo.push(edit.clone());
        Some(edit)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
}
