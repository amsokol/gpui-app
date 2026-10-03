# gpui-app

Desktop app built with `gpui-kit` v0.7.0 (single dependency; GPUI is `use gpui_kit::*;`).

## Language

Always answer the user in Russian. Keep code, identifiers, commands, commit messages and project
files (README, docs) in English unless the user asks otherwise.

## Skills

Before any UI work, read the `gpui-kit-design-guides` skill. Before any architecture, state-ownership,
public-API, naming or testing decision, read the Coding Guides in the `gpui-kit` skill. Never invent
an API: check the skill references or `https://gpui-kit.com/component/{name}.md` for the real signature.

The skills are installed in `.agents/skills/` (see `skills-lock.json`) and symlinked from
`.claude/skills/`. Update them with `npx skills add longbridge/gpui-kit`.

## Build and run

- `cargo run` — needs `libxkbcommon-x11-dev` on Debian/Ubuntu.
- Startup is `gpui_kit::init(cx)` then `gpui_kit::open_window(...)`, which wraps the view in `Root`
  automatically. Do not call `Root::new` or the removed `Root::render_*_layer` APIs.

## Environment notes (WSL2 / WSLg)

- GPUI renders on the CPU (lavapipe, `PHYSICAL_DEVICE_TYPE_CPU`) because WSL has no hardware Vulkan
  driver, so window moves and resizes are slow. This is an environment limit, not an app bug.
- The window has no compositor frame, so `TitleBar` (gpui-kit) draws the title bar and controls.
- Oversized mouse cursor at 200% display scaling is a WSLg bug. Fix:
  `gsettings set org.gnome.desktop.interface cursor-size 12` (see README).
- Do not use the X11 backend (`WAYLAND_DISPLAY=`) as a workaround: text is blurry and WSLg adds a
  second title bar.
