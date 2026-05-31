use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

/// Keyboard + mouse input used by gameplay systems (counts as one system param).
#[derive(SystemParam)]
pub struct GameInput<'w> {
    pub keys: Res<'w, ButtonInput<KeyCode>>,
    pub mouse: Res<'w, ButtonInput<MouseButton>>,
    pub bindings: Res<'w, KeyBindings>,
}

/// Identifies a rebindable action shown in Settings → Keybindings.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum BindingId {
    ModePlace,
    ModeSelect,
    ModePaint,
    TogglePlaceSelect,
    RotateCcw,
    RotateCw,
    Undo,
    Redo,
    Delete,
}

impl BindingId {
    pub const ALL: [BindingId; 9] = [
        BindingId::ModePlace,
        BindingId::ModeSelect,
        BindingId::ModePaint,
        BindingId::TogglePlaceSelect,
        BindingId::RotateCcw,
        BindingId::RotateCw,
        BindingId::Undo,
        BindingId::Redo,
        BindingId::Delete,
    ];

    pub fn label(self) -> &'static str {
        match self {
            BindingId::ModePlace => "Place mode",
            BindingId::ModeSelect => "Select mode",
            BindingId::ModePaint => "Paint mode",
            BindingId::TogglePlaceSelect => "Toggle Place / Select",
            BindingId::RotateCcw => "Rotate selection / placement (−90° Y)",
            BindingId::RotateCw => "Rotate selection / placement (+90° Y)",
            BindingId::Undo => "Undo",
            BindingId::Redo => "Redo",
            BindingId::Delete => "Delete selection",
        }
    }

    pub fn hint(self) -> &'static str {
        match self {
            BindingId::Undo => "Hold Ctrl",
            BindingId::Redo => "Hold Ctrl",
            BindingId::Delete => "Also Backspace",
            _ => "",
        }
    }
}

#[derive(Resource, Clone)]
pub struct KeyBindings {
    pub mode_place: KeyCode,
    pub mode_select: KeyCode,
    pub mode_paint: KeyCode,
    pub toggle_place_select: KeyCode,
    pub rotate_ccw: KeyCode,
    pub rotate_cw: KeyCode,
    pub undo: KeyCode,
    pub redo: KeyCode,
    pub delete: KeyCode,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            mode_place: KeyCode::Digit1,
            mode_select: KeyCode::Digit2,
            mode_paint: KeyCode::Digit3,
            toggle_place_select: KeyCode::Tab,
            rotate_ccw: KeyCode::KeyQ,
            rotate_cw: KeyCode::KeyE,
            undo: KeyCode::KeyZ,
            redo: KeyCode::KeyY,
            delete: KeyCode::Delete,
        }
    }
}

impl KeyBindings {
    pub fn get(self: &Self, id: BindingId) -> KeyCode {
        match id {
            BindingId::ModePlace => self.mode_place,
            BindingId::ModeSelect => self.mode_select,
            BindingId::ModePaint => self.mode_paint,
            BindingId::TogglePlaceSelect => self.toggle_place_select,
            BindingId::RotateCcw => self.rotate_ccw,
            BindingId::RotateCw => self.rotate_cw,
            BindingId::Undo => self.undo,
            BindingId::Redo => self.redo,
            BindingId::Delete => self.delete,
        }
    }

    pub fn set(&mut self, id: BindingId, key: KeyCode) {
        match id {
            BindingId::ModePlace => self.mode_place = key,
            BindingId::ModeSelect => self.mode_select = key,
            BindingId::ModePaint => self.mode_paint = key,
            BindingId::TogglePlaceSelect => self.toggle_place_select = key,
            BindingId::RotateCcw => self.rotate_ccw = key,
            BindingId::RotateCw => self.rotate_cw = key,
            BindingId::Undo => self.undo = key,
            BindingId::Redo => self.redo = key,
            BindingId::Delete => self.delete = key,
        }
    }

    pub fn key_label(key: KeyCode) -> String {
        match key {
            KeyCode::Digit0 => "0".into(),
            KeyCode::Digit1 => "1".into(),
            KeyCode::Digit2 => "2".into(),
            KeyCode::Digit3 => "3".into(),
            KeyCode::Digit4 => "4".into(),
            KeyCode::Digit5 => "5".into(),
            KeyCode::Digit6 => "6".into(),
            KeyCode::Digit7 => "7".into(),
            KeyCode::Digit8 => "8".into(),
            KeyCode::Digit9 => "9".into(),
            KeyCode::Tab => "Tab".into(),
            KeyCode::Delete => "Delete".into(),
            KeyCode::Backspace => "Backspace".into(),
            KeyCode::Escape => "Esc".into(),
            KeyCode::Space => "Space".into(),
            other => format!("{other:?}"),
        }
    }

    pub fn just_pressed(&self, keys: &ButtonInput<KeyCode>, id: BindingId) -> bool {
        keys.just_pressed(self.get(id))
    }

    pub fn delete_pressed(&self, keys: &ButtonInput<KeyCode>) -> bool {
        keys.just_pressed(self.delete) || keys.just_pressed(KeyCode::Backspace)
    }

    pub fn ctrl_pressed(keys: &ButtonInput<KeyCode>) -> bool {
        keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight)
    }

    pub fn shift_pressed(keys: &ButtonInput<KeyCode>) -> bool {
        keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight)
    }

    pub fn alt_pressed(keys: &ButtonInput<KeyCode>) -> bool {
        keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight)
    }

    /// First key pressed this frame (for rebinding UI). Returns `None` on Escape (cancel).
    pub fn capture_rebind_key(keys: &ButtonInput<KeyCode>) -> Option<KeyCode> {
        if keys.just_pressed(KeyCode::Escape) {
            return None;
        }
        REBINDABLE_KEYS
            .iter()
            .copied()
            .find(|k| keys.just_pressed(*k))
    }
}

/// Keys users can assign in Settings (keep list small and sensible).
const REBINDABLE_KEYS: &[KeyCode] = &[
    KeyCode::Digit0,
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
    KeyCode::Digit6,
    KeyCode::Digit7,
    KeyCode::Digit8,
    KeyCode::Digit9,
    KeyCode::KeyA,
    KeyCode::KeyB,
    KeyCode::KeyC,
    KeyCode::KeyD,
    KeyCode::KeyE,
    KeyCode::KeyF,
    KeyCode::KeyG,
    KeyCode::KeyH,
    KeyCode::KeyI,
    KeyCode::KeyJ,
    KeyCode::KeyK,
    KeyCode::KeyL,
    KeyCode::KeyM,
    KeyCode::KeyN,
    KeyCode::KeyO,
    KeyCode::KeyP,
    KeyCode::KeyQ,
    KeyCode::KeyR,
    KeyCode::KeyS,
    KeyCode::KeyT,
    KeyCode::KeyU,
    KeyCode::KeyV,
    KeyCode::KeyW,
    KeyCode::KeyX,
    KeyCode::KeyY,
    KeyCode::KeyZ,
    KeyCode::Tab,
    KeyCode::Space,
    KeyCode::Delete,
    KeyCode::Backspace,
];
