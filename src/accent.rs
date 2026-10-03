//! The system accent color, used in place of the blue that is baked into the macOS Classic themes.
//!
//! The color comes from the `mundy` crate: the XDG Settings portal on Linux (with a Yaru theme
//! fallback on Ubuntu), `UISettings` on Windows and `NSColor.controlAccentColor` on macOS. When
//! the system does not report one, the theme keeps its own color.

use std::{env, time::Duration};

use futures_lite::StreamExt as _;
use gpui_kit::component::ThemeConfig;
use gpui_kit::{App, Global};
use mundy::{Interest, Preferences};

/// Environment variable that forces an accent color, e.g. `GPUI_APP_ACCENT=#e95420`. It is a way
/// to try the tinting on a system that reports no accent color (such as WSL).
const OVERRIDE_ENV: &str = "GPUI_APP_ACCENT";

/// How long to wait for the system at startup. The window opens afterwards, so keep it short.
const READ_TIMEOUT: Duration = Duration::from_millis(200);

/// Below this contrast ratio against white, text on the accent color turns black.
const MIN_CONTRAST_WITH_WHITE: f64 = 3.0;

/// An sRGB color with 8 bits per channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    /// Parses `#rrggbb` or `rrggbb`.
    fn parse(text: &str) -> Option<Self> {
        let digits = text.trim().trim_start_matches('#');
        if digits.len() != 6 || !digits.is_ascii() {
            return None;
        }
        let channel = |range: std::ops::Range<usize>| u8::from_str_radix(&digits[range], 16).ok();
        Some(Rgb(channel(0..2)?, channel(2..4)?, channel(4..6)?))
    }

    fn hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
    }

    /// Hex with an alpha channel, used for translucent highlights.
    fn hex_alpha(self, alpha: u8) -> String {
        format!("{}{alpha:02x}", self.hex())
    }

    /// WCAG relative luminance.
    fn luminance(self) -> f64 {
        let linear = |channel: u8| {
            let c = f64::from(channel) / 255.;
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(self.0) + 0.7152 * linear(self.1) + 0.0722 * linear(self.2)
    }

    /// White text on the color, or black when the color is too light for white to read.
    fn foreground(self) -> Rgb {
        if 1.05 / (self.luminance() + 0.05) >= MIN_CONTRAST_WITH_WHITE {
            Rgb(255, 255, 255)
        } else {
            Rgb(0, 0, 0)
        }
    }
}

impl From<mundy::Srgba> for Rgb {
    fn from(color: mundy::Srgba) -> Self {
        let channel = |value: f64| (value.clamp(0., 1.) * 255.).round() as u8;
        Rgb(
            channel(color.red),
            channel(color.green),
            channel(color.blue),
        )
    }
}

/// The accent color currently reported by the system, kept as an app-wide global.
#[derive(Default)]
struct SystemAccent(Option<Rgb>);

impl Global for SystemAccent {}

/// The accent color last reported by the system.
pub fn current(cx: &App) -> Option<Rgb> {
    cx.try_global::<SystemAccent>().and_then(|accent| accent.0)
}

/// The color forced through [`OVERRIDE_ENV`], if it is set to a valid color.
fn forced() -> Option<Rgb> {
    env::var(OVERRIDE_ENV)
        .ok()
        .and_then(|text| Rgb::parse(&text))
}

/// Reads the accent color once and stores it. Call before opening the window, on the main thread.
pub fn init(cx: &mut App) {
    let accent = forced().or_else(|| {
        Preferences::once_blocking(Interest::AccentColor, READ_TIMEOUT)
            .and_then(|preferences| preferences.accent_color.0)
            .map(Rgb::from)
    });
    cx.set_global(SystemAccent(accent));
}

/// Follows accent color changes and calls `on_change` after each one.
pub fn watch(cx: &mut App, on_change: impl Fn(&mut App) + 'static) {
    if forced().is_some() {
        return;
    }
    cx.spawn(async move |cx| {
        let mut stream = Preferences::stream(Interest::AccentColor);
        while let Some(preferences) = stream.next().await {
            let accent = preferences.accent_color.0.map(Rgb::from);
            cx.update(|cx| {
                if current(cx) != accent {
                    cx.set_global(SystemAccent(accent));
                    on_change(cx);
                }
            });
        }
    })
    .detach();
}

/// Replaces the blue of a macOS Classic theme with `accent`.
pub fn tint(theme: &mut ThemeConfig, accent: Rgb) {
    let colors = &mut theme.colors;
    colors.primary = Some(accent.hex().into());
    colors.primary_foreground = Some(accent.foreground().hex().into());
    colors.ring = Some(accent.hex().into());
    colors.list_active = Some(accent.hex_alpha(0x15).into());
    colors.list_active_border = Some(accent.hex().into());
    // Only the dark theme sets its own text selection color; the light one derives it.
    if colors.selection.is_some() {
        colors.selection = Some(accent.hex_alpha(0x66).into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_srgba_to_bytes() {
        let color = mundy::Srgba {
            red: 1.,
            green: 0.5,
            blue: 0.,
            alpha: 1.,
        };
        assert_eq!(Rgb::from(color), Rgb(255, 128, 0));
    }

    #[test]
    fn parses_hex() {
        assert_eq!(Rgb::parse("#e95420"), Some(Rgb(0xE9, 0x54, 0x20)));
        assert_eq!(Rgb::parse(" 0060DE "), Some(Rgb(0x00, 0x60, 0xDE)));
        assert_eq!(Rgb::parse("#e954"), None);
        assert_eq!(Rgb::parse("#gg0000"), None);
    }

    #[test]
    fn formats_hex() {
        assert_eq!(Rgb(0xE9, 0x54, 0x20).hex(), "#e95420");
        assert_eq!(Rgb(0xE9, 0x54, 0x20).hex_alpha(0x15), "#e9542015");
    }

    #[test]
    fn picks_readable_foreground() {
        // Ubuntu orange and macOS blue take white text, a light yellow takes black.
        assert_eq!(Rgb(0xE9, 0x54, 0x20).foreground(), Rgb(255, 255, 255));
        assert_eq!(Rgb(0x00, 0x60, 0xDE).foreground(), Rgb(255, 255, 255));
        assert_eq!(Rgb(0xF5, 0xC2, 0x11).foreground(), Rgb(0, 0, 0));
    }

    #[test]
    fn tint_overrides_accent_keys() {
        let mut theme = ThemeConfig::default();
        tint(&mut theme, Rgb(0xE9, 0x54, 0x20));
        assert_eq!(theme.colors.primary.as_deref(), Some("#e95420"));
        assert_eq!(theme.colors.list_active.as_deref(), Some("#e9542015"));
        // Left alone when the theme does not set it.
        assert_eq!(theme.colors.selection, None);
    }
}
