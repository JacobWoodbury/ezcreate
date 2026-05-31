use bevy::prelude::*;

use super::KeyBindings;

/// Which high-level UI screen is active.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub enum AppScreen {
    LaunchMenu,
    Ftue { step: usize },
    Playing,
}

impl Default for AppScreen {
    fn default() -> Self {
        Self::LaunchMenu
    }
}

impl AppScreen {
    pub fn enter_playing(screen: &mut Self) {
        *screen = AppScreen::Playing;
    }

    pub fn start_ftue(screen: &mut Self) {
        *screen = AppScreen::Ftue { step: 0 };
    }

    pub fn ftue_next(screen: &mut Self) {
        if let AppScreen::Ftue { step } = screen {
            *step += 1;
            if *step >= FTUE_STEPS.len() {
                Self::enter_playing(screen);
            }
        }
    }

    pub fn ftue_back(screen: &mut Self) {
        if let AppScreen::Ftue { step } = screen {
            *step = step.saturating_sub(1);
        }
    }

    pub fn ftue_skip(screen: &mut Self) {
        Self::enter_playing(screen);
    }
}

pub struct FtueStep {
    pub title: &'static str,
    pub body: fn(&KeyBindings) -> String,
}

pub const FTUE_STEPS: &[FtueStep] = &[
    FtueStep {
        title: "Welcome to ezcreate",
        body: |_| {
            "Build modular structures on a grid using blocks from the library.\n\n\
             The interface has three main areas:\n\
             • Top bar — switch Place, Select, and Paint modes\n\
             • Left sidebar — block library and mode-specific tools\n\
             • 3D view — your build area with a ground plane\n\n\
             Use the buttons below to step through the controls."
                .into()
        },
    },
    FtueStep {
        title: "Place mode",
        body: |b| {
            format!(
                "Press {} or click Place in the top bar.\n\n\
                 1. Click a block in the Library sidebar\n\
                 2. Move the mouse over the ground — a green ghost shows placement\n\
                 3. Left-click to place\n\
                 4. Press {} / {} to rotate before placing\n\n\
                 Toggle \"Prevent overlap\" in the top bar to allow stacking on occupied cells.",
                KeyBindings::key_label(b.mode_place),
                KeyBindings::key_label(b.rotate_ccw),
                KeyBindings::key_label(b.rotate_cw),
            )
        },
    },
    FtueStep {
        title: "Select mode",
        body: |b| {
            format!(
                "Press {} or click Select in the top bar.\n\n\
                 • Click a block to select it\n\
                 • Drag a rectangle to marquee-select several blocks\n\
                 • Hold Shift while clicking to add/remove from selection\n\
                 • Press {} to delete the selection\n\
                 • {} / {} rotate the whole selection\n\n\
                 Save a selection as a reusable module with \"Save selection as module\" in the sidebar.",
                KeyBindings::key_label(b.mode_select),
                KeyBindings::key_label(b.delete),
                KeyBindings::key_label(b.rotate_ccw),
                KeyBindings::key_label(b.rotate_cw),
            )
        },
    },
    FtueStep {
        title: "Paint mode",
        body: |b| {
            format!(
                "Press {} or click Paint in the top bar.\n\n\
                 In the Stamp editor sidebar:\n\
                 • Pick a brush color and edit the pixel grid\n\
                 • Switch to Apply mode to paint on block faces\n\
                 • Hover a face for a preview, then left-click to apply\n\
                 • Clear the grid to use a solid brush color instead\n\
                 • Name and Save stamps for reuse later\n\n\
                 The \"Painted faces\" list lets you remove decals from selected blocks.",
                KeyBindings::key_label(b.mode_paint),
            )
        },
    },
    FtueStep {
        title: "Undo and settings",
        body: |b| {
            format!(
                "• Ctrl+{} — undo placement, deletion, and paint\n\
                 • Ctrl+{} — redo\n\
                 • Alt + right-click — delete the block under the cursor (any mode)\n\n\
                 Open Settings (gear icon, top-right) for:\n\
                 • Grid size, overlap rules, camera pan direction\n\
                 • Keybindings — rebind any action\n\n\
                 Press {} to quickly toggle Place and Select.",
                KeyBindings::key_label(b.undo),
                KeyBindings::key_label(b.redo),
                KeyBindings::key_label(b.toggle_place_select),
            )
        },
    },
    FtueStep {
        title: "Camera",
        body: |_| {
            "Navigate the 3D view with:\n\n\
             • W A S D — pan across the ground\n\
             • Right-click and drag — orbit around the scene\n\
             • Scroll wheel — zoom in and out\n\n\
             Invert W/S pan direction under Settings → General if it feels reversed."
                .into()
        },
    },
    FtueStep {
        title: "You're ready to build",
        body: |_| {
            "That covers the basics.\n\n\
             Pick a block from the library, place your first structure, and experiment \
             with selection, paint, and saved modules.\n\n\
             Reopen this tutorial anytime from Help in the top bar."
                .into()
        },
    },
];
