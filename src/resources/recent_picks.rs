use bevy::prelude::*;

use crate::content::LibraryItemRef;

const MAX_RECENT: usize = 12;

#[derive(Resource, Default)]
pub struct RecentPicks {
    pub items: Vec<LibraryItemRef>,
}

impl RecentPicks {
    pub fn push(&mut self, item: LibraryItemRef) {
        self.items.retain(|i| i.item_id != item.item_id);
        self.items.insert(0, item);
        self.items.truncate(MAX_RECENT);
    }
}
