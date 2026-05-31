use bevy::prelude::*;
use std::collections::HashSet;

#[derive(Resource, Default)]
pub struct SelectionState {
    pub selected: HashSet<Entity>,
    pub marquee_dragging: bool,
    pub marquee_start: Option<Vec2>,
    pub marquee_current: Option<Vec2>,
}

impl SelectionState {
    pub fn clear(&mut self) {
        self.selected.clear();
    }

    pub fn toggle(&mut self, entity: Entity) {
        if !self.selected.insert(entity) {
            self.selected.remove(&entity);
        }
    }

    pub fn set_single(&mut self, entity: Entity) {
        self.selected.clear();
        self.selected.insert(entity);
    }

    pub fn marquee_rect(&self) -> Option<Rect> {
        let start = self.marquee_start?;
        let end = self.marquee_current?;
        let min = start.min(end);
        let max = start.max(end);
        Some(Rect::from_corners(min, max))
    }

    pub fn marquee_drag_distance(&self) -> f32 {
        match (self.marquee_start, self.marquee_current) {
            (Some(a), Some(b)) => a.distance(b),
            _ => 0.0,
        }
    }
}
