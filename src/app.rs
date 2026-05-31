use crate::{
    content::ContentPlugin,
    resources::{GameMode, GameModeChanged, OccupancyMap, PlacementState, UndoStack},
    systems::{
        camera_orbit::OrbitCameraPlugin, input_router::InputRouterPlugin,
        placement::PlacementPlugin, undo_redo::UndoRedoPlugin, world_setup::WorldSetupPlugin,
    },
    ui::UiPlugin,
};
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;

pub struct EzCreatePlugin;

impl Plugin for EzCreatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameMode>()
            .init_resource::<crate::resources::GamePreferences>()
            .init_resource::<OccupancyMap>()
            .init_resource::<PlacementState>()
            .init_resource::<UndoStack>()
            .add_message::<GameModeChanged>()
            .add_plugins((
                EguiPlugin::default(),
                PhysicsPlugins::default(),
                WorldSetupPlugin,
                OrbitCameraPlugin,
                ContentPlugin,
                PlacementPlugin,
                InputRouterPlugin,
                UndoRedoPlugin,
                UiPlugin,
            ));
    }
}
