# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added

- Simple desktop app built with [gpui-kit](https://github.com/longbridge/gpui-kit) v0.7.0:
  text input with a live greeting, "Shout" switch, click counter with Reset, and an About dialog.
- Custom window title bar (`TitleBar`) with window controls, since WSLg draws no window frame.
- Light/dark theme follows the system setting and updates live
  (`Theme::sync_system_appearance` plus `observe_window_appearance`).
- "macOS Classic Light" and "macOS Classic Dark" themes (`themes/macos-classic.json`, by huacnlee,
  from gpui-kit), embedded in the binary and chosen by the system light/dark setting.
- Window size, position and maximized state are saved as YAML on close and restored on start
  in the per-user local config directory (`dirs` crate): `~/.config/gpui-app/window.yaml` on Linux,
  `~/Library/Application Support/gpui-app/window.yaml` on macOS,
  `%LOCALAPPDATA%\gpui-app\window.yaml` on Windows.
- No flash of the wrong theme at startup on Linux: Wayland reports the system light/dark mode only
  after startup, so the window is opened once the mode is known (waits up to 100 ms), and later
  system events are applied after a short pause because GPUI first reports a bogus "light".
- `release` profile with full optimization (`opt-level = 3`, fat LTO, one codegen unit, stripped
  symbols) and a statically linked C runtime on Windows (MSVC).
- README with run instructions, the `libxkbcommon-x11-dev` build dependency, and WSL notes
  (oversized mouse cursor at 200% display scaling and how to fix it).
- gpui-kit agent skills (`gpui-kit`, `gpui-kit-design-guides`) installed in `.agents/skills/`
  and symlinked from `.claude/skills/`.
- `CLAUDE.md` with project instructions for Claude Code: reply language, skills usage,
  build notes, and WSL environment notes.

[Unreleased]: https://github.com/amsokol/gpui-app/commits/main
