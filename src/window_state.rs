//! Saves and restores the window size, position and maximized state as YAML.
//!
//! The file lives in the per-user local config directory (`dirs::config_local_dir`):
//!
//! - Linux: `~/.config/gpui-app/window.yaml` (`$XDG_CONFIG_HOME` is honored)
//! - macOS: `~/Library/Application Support/gpui-app/window.yaml`
//! - Windows: `%LOCALAPPDATA%\gpui-app\window.yaml`
//!
//! Wayland compositors choose where a window goes, so there only the size and the maximized
//! state are restored; the saved position is ignored.

use std::{fs, path::PathBuf};

use gpui_kit::{App, Bounds, WindowBounds, point, px, size};
use serde::{Deserialize, Serialize};

const APP_DIR: &str = "gpui-app";
const FILE_NAME: &str = "window.yaml";

/// Smallest size accepted from the file, so a damaged file cannot open an unusable window.
const MIN_WIDTH: f32 = 320.;
const MIN_HEIGHT: f32 = 240.;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WindowState {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub maximized: bool,
}

impl WindowState {
    /// Captures the current window bounds. A maximized window keeps its restore size, and a
    /// fullscreen window is saved as a normal one.
    pub fn from_window_bounds(bounds: WindowBounds) -> Self {
        let maximized = matches!(bounds, WindowBounds::Maximized(_));
        let bounds = bounds.get_bounds();
        Self {
            x: f32::from(bounds.origin.x),
            y: f32::from(bounds.origin.y),
            width: f32::from(bounds.size.width),
            height: f32::from(bounds.size.height),
            maximized,
        }
    }

    /// Loads the saved state. A missing, unreadable or invalid file gives `None`.
    pub fn load() -> Option<Self> {
        let path = config_path()?;
        let content = fs::read_to_string(&path).ok()?;
        match yaml_serde::from_str::<Self>(&content) {
            Ok(state) if state.is_valid() => Some(state),
            Ok(_) => {
                eprintln!("ignoring invalid window state in {}", path.display());
                None
            }
            Err(err) => {
                eprintln!("ignoring unreadable {}: {err}", path.display());
                None
            }
        }
    }

    /// Writes the state, going through a temporary file so a crash cannot leave half a file.
    pub fn save(&self) {
        if let Err(err) = self.try_save() {
            eprintln!("failed to save window state: {err}");
        }
    }

    /// Converts to window bounds for `WindowOptions`. If the saved position is not on any
    /// connected display (for example a monitor was unplugged), the window is centered instead.
    pub fn to_window_bounds(self, cx: &App) -> WindowBounds {
        let window_size = size(px(self.width), px(self.height));
        let bounds = Bounds::new(point(px(self.x), px(self.y)), window_size);

        let on_screen = cx
            .displays()
            .iter()
            .any(|display| display.bounds().contains(&bounds.center()));
        let bounds = if on_screen {
            bounds
        } else {
            Bounds::centered(None, window_size, cx)
        };

        if self.maximized {
            WindowBounds::Maximized(bounds)
        } else {
            WindowBounds::Windowed(bounds)
        }
    }

    fn is_valid(&self) -> bool {
        [self.x, self.y, self.width, self.height]
            .iter()
            .all(|value| value.is_finite())
            && self.width >= MIN_WIDTH
            && self.height >= MIN_HEIGHT
    }

    fn try_save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = config_path().ok_or("no home or config directory")?;
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        let temp = path.with_extension("yaml.tmp");
        fs::write(&temp, yaml_serde::to_string(self)?)?;
        fs::rename(&temp, &path)?;
        Ok(())
    }
}

fn config_path() -> Option<PathBuf> {
    dirs::config_local_dir().map(|dir| dir.join(APP_DIR).join(FILE_NAME))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_yaml() {
        let state = WindowState {
            x: 120.,
            y: 80.5,
            width: 900.,
            height: 700.,
            maximized: true,
        };
        let yaml = yaml_serde::to_string(&state).unwrap();
        assert_eq!(yaml_serde::from_str::<WindowState>(&yaml).unwrap(), state);
    }

    #[test]
    fn rejects_tiny_or_non_finite_sizes() {
        let valid = WindowState {
            x: 0.,
            y: 0.,
            width: 800.,
            height: 600.,
            maximized: false,
        };
        assert!(valid.is_valid());
        assert!(
            !WindowState {
                width: 10.,
                ..valid
            }
            .is_valid()
        );
        assert!(
            !WindowState {
                height: f32::NAN,
                ..valid
            }
            .is_valid()
        );
    }
}
