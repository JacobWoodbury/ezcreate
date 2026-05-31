use bevy::prelude::*;

#[derive(Clone)]
pub struct PlacedBlockSnapshot {
    pub item_id: String,
    pub grid_key: IVec3,
    pub rotation: Quat,
}

#[derive(Clone)]
pub enum GridEdit {
    Place {
        snapshot: PlacedBlockSnapshot,
    },
    Delete {
        snapshot: PlacedBlockSnapshot,
    },
}

#[derive(Resource, Default)]
pub struct UndoStack {
    undo: Vec<GridEdit>,
    redo: Vec<GridEdit>,
}

impl UndoStack {
    pub fn push(&mut self, edit: GridEdit) {
        self.undo.push(edit);
        self.redo.clear();
    }

    pub fn pop_undo(&mut self) -> Option<GridEdit> {
        let edit = self.undo.pop()?;
        self.redo.push(edit.clone());
        Some(edit)
    }

    pub fn pop_redo(&mut self) -> Option<GridEdit> {
        let edit = self.redo.pop()?;
        self.undo.push(edit.clone());
        Some(edit)
    }
}
