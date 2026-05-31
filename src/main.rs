mod app;
mod components;
mod content;
mod resources;
mod systems;
mod ui;

use app::EzCreatePlugin;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "ezcreate".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EzCreatePlugin)
        .run();
}
