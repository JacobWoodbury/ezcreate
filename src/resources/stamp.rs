use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const DEFAULT_STAMP_SIZE: usize = 4;

/// A 2-D pixel grid used for face stamping in Paint mode.
#[derive(Clone, Serialize, Deserialize)]
pub struct Stamp {
    pub width: usize,
    pub height: usize,
    /// RGBA pixels, row-major (top-left first), length = width * height.
    pub pixels: Vec<[u8; 4]>,
}

impl Default for Stamp {
    fn default() -> Self {
        let n = DEFAULT_STAMP_SIZE;
        Self {
            width: n,
            height: n,
            pixels: vec![[180, 180, 180, 220]; n * n],
        }
    }
}

impl Stamp {
    pub fn get(&self, col: usize, row: usize) -> [u8; 4] {
        self.pixels[row * self.width + col]
    }

    pub fn set(&mut self, col: usize, row: usize, color: [u8; 4]) {
        self.pixels[row * self.width + col] = color;
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        let mut new_pixels = vec![[180u8, 180, 180, 220]; width * height];
        let copy_w = self.width.min(width);
        let copy_h = self.height.min(height);
        for r in 0..copy_h {
            for c in 0..copy_w {
                new_pixels[r * width + c] = self.get(c, r);
            }
        }
        self.width = width;
        self.height = height;
        self.pixels = new_pixels;
    }
}

/// Runtime state for the stamp painter in Paint mode.
#[derive(Resource)]
pub struct StampPainter {
    /// The stamp being edited / applied.
    pub stamp: Stamp,
    /// Active brush color used when clicking stamp grid cells.
    pub brush_color: [u8; 4],
    /// Stamps loaded from the user stamps directory.
    pub saved_stamps: Vec<(String, Stamp)>,
    /// True while the user is in "apply" sub-mode (clicking blocks).
    /// False = editing the stamp grid; True = applying to faces.
    pub apply_mode: bool,
}

impl Default for StampPainter {
    fn default() -> Self {
        Self {
            stamp: Stamp::default(),
            brush_color: [220, 80, 60, 255],
            saved_stamps: Vec::new(),
            apply_mode: true,
        }
    }
}

impl StampPainter {
    /// True when every pixel is fully transparent (use solid brush color on faces).
    pub fn stamp_is_empty(&self) -> bool {
        self.stamp.pixels.iter().all(|p| p[3] < 16)
    }

    /// True when the stamp grid should be applied to faces (any opaque pixel).
    pub fn apply_uses_stamp_grid(&self) -> bool {
        !self.stamp_is_empty()
    }

    pub fn brush_color_bevy(&self) -> Color {
        let [r, g, b, a] = self.brush_color;
        Color::srgba(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        )
    }

    /// Directory for user-created stamps.
    pub fn stamps_dir() -> Option<PathBuf> {
        dirs::data_dir().map(|d| d.join("ezcreate/stamps"))
    }

    /// Save the current stamp to disk and push to `saved_stamps`.
    pub fn save_stamp(&mut self, name: &str) -> Result<(), String> {
        let dir = Self::stamps_dir().ok_or("Cannot resolve user data directory.")?;
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let filename = format!("{}.json", sanitize_filename(name));
        let path = dir.join(&filename);
        let json = serde_json::to_string_pretty(&self.stamp).map_err(|e| e.to_string())?;
        std::fs::write(&path, json).map_err(|e| e.to_string())?;
        self.saved_stamps.push((name.to_string(), self.stamp.clone()));
        Ok(())
    }

    /// Scan the stamps directory and populate `saved_stamps`.
    pub fn reload_stamps(&mut self) {
        self.saved_stamps.clear();
        let Some(dir) = Self::stamps_dir() else {
            return;
        };
        let Ok(entries) = std::fs::read_dir(&dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Stamp")
                .to_string();
            if let Ok(text) = std::fs::read_to_string(&path) {
                if let Ok(stamp) = serde_json::from_str::<Stamp>(&text) {
                    self.saved_stamps.push((name, stamp));
                }
            }
        }
    }
}

fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}
