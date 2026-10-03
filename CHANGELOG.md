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
- README with run instructions, the `libxkbcommon-x11-dev` build dependency, and WSL notes
  (oversized mouse cursor at 200% display scaling and how to fix it).
- gpui-kit agent skills (`gpui-kit`, `gpui-kit-design-guides`) installed in `.agents/skills/`
  and symlinked from `.claude/skills/`.
- `CLAUDE.md` with project instructions for Claude Code: reply language, skills usage,
  build notes, and WSL environment notes.
