use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::Stamp;

#[derive(Clone)]
pub struct PlacedBlockSnapshot {
    pub item_id: String,
    pub grid_key: IVec3,
    pub rotation: Quat,
    pub scene_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "paintType", rename_all = "camelCase")]
pub enum FacePaintKind {
    Solid,
    Stamp {
        stamp: Stamp,
        #[serde(default)]
        rotation_quarters: i32,
    },
}

#[derive(Clone)]
pub struct FacePaintSnapshot {
    pub parent_block: Entity,
    pub color: Color,
    pub face_normal: Vec3,
    pub face_size: f32,
    pub bias: f32,
    pub stamp_rotation_quarters: i32,
    pub kind: FacePaintKind,
}

#[derive(Clone)]
pub enum GridEdit {
    Place {
        snapshot: PlacedBlockSnapshot,
    },
    Delete {
        snapshot: PlacedBlockSnapshot,
    },
    /// Undo for section placement (all pieces placed together).
    BulkPlace {
        snapshots: Vec<PlacedBlockSnapshot>,
    },
    BulkDelete {
        snapshots: Vec<PlacedBlockSnapshot>,
    },
    FacePaint {
        snapshot: FacePaintSnapshot,
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
