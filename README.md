# gpui-app

A simple desktop app built with [gpui-kit](https://github.com/longbridge/gpui-kit) v0.7.0.

## Run

```sh
cargo run
```

### Release build

```sh
cargo build --release
```

The `release` profile uses full optimization (`opt-level = 3`, fat LTO, one codegen unit) and strips
symbols: about 31 MB instead of 700 MB for a debug build, and about 4 minutes to build.

On Windows (MSVC) the C runtime is linked statically (`.cargo/config.toml`), so the `.exe` needs no
Visual C++ Redistributable. On Linux the binary is dynamically linked against glibc, xcb and
xkbcommon, which every desktop distribution has. A fully static Linux binary (musl) is not possible:
GPUI loads Vulkan, Wayland and font libraries at run time.

### Linux build dependencies

Linking needs the X11 xkbcommon development files. On Debian/Ubuntu:

```sh
sudo apt install libxkbcommon-x11-dev
```

## Window state

The window size, position and maximized state are saved when the window closes and restored on the
next start:

- Linux: `~/.config/gpui-app/window.yaml` (`$XDG_CONFIG_HOME` is honored)
- macOS: `~/Library/Application Support/gpui-app/window.yaml`
- Windows: `%LOCALAPPDATA%\gpui-app\window.yaml`

Delete the file to go back to the default window. Wayland compositors (including WSLg) choose the
window position themselves, so there only the size and the maximized state are restored.

## WSL notes

### Mouse cursor is too large

On WSL2 (WSLg) with Windows display scaling above 100% (e.g. 200%), the mouse
cursor can look about twice as big as normal over the app window. GPUI sends a
correctly scaled cursor image, but WSLg's compositor does not honour the
buffer scale for cursor surfaces.

Halve the cursor size to compensate. The default is `24`; at 200% scaling use `12`:

```sh
gsettings set org.gnome.desktop.interface cursor-size 12
```

Restart the app afterwards. To undo:

```sh
gsettings reset org.gnome.desktop.interface cursor-size
```

This setting affects every Wayland app in the WSL distro, not just this one.

### Dark theme

The app follows the system light/dark setting and switches live. In WSL, set it with:

```sh
gsettings set org.gnome.desktop.interface color-scheme 'prefer-dark'
```

Use `'default'` to go back to light. This affects every GTK/Wayland app in the distro.

### Don't use the X11 backend to work around it

Running with `WAYLAND_DISPLAY=` gives a normal cursor, but WSLg's X11 windows
are not DPI-aware, so the text is blurry and WSLg adds a second title bar.
