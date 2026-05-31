use crate::{
    content::ContentPlugin,
    resources::{
        GameMode, GameModeChanged, KeyBindings, OccupancyMap, PaintState, PlacementState,
        RecentPicks, SelectionState, UndoStack, AppScreen,
    },
    systems::{
        camera_orbit::OrbitCameraPlugin, input_router::InputRouterPlugin,
        paint::PaintPlugin, placement::PlacementPlugin, selection::SelectionPlugin,
        thumbnails::ThumbnailPlugin, undo_redo::UndoRedoPlugin, world_setup::WorldSetupPlugin,
    },
    ui::{configure_gameplay_after_ui, UiPlugin},
};
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;

pub struct EzCreatePlugin;

impl Plugin for EzCreatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameMode>()
            .init_resource::<AppScreen>()
            .init_resource::<crate::resources::GamePreferences>()
            .init_resource::<KeyBindings>()
            .init_resource::<OccupancyMap>()
            .init_resource::<PlacementState>()
            .init_resource::<SelectionState>()
            .init_resource::<PaintState>()
            .init_resource::<RecentPicks>()
            .init_resource::<UndoStack>()
            .add_message::<GameModeChanged>();
        configure_gameplay_after_ui(app);
        app.add_plugins((
                EguiPlugin::default(),
                PhysicsPlugins::default(),
                WorldSetupPlugin,
                OrbitCameraPlugin,
                ContentPlugin,
                PlacementPlugin,
                SelectionPlugin,
                PaintPlugin,
                ThumbnailPlugin,
                InputRouterPlugin,
                UndoRedoPlugin,
                UiPlugin,
            ));
    }
}
