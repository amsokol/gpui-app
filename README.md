# gpui-app

A simple desktop app built with [gpui-kit](https://github.com/longbridge/gpui-kit) v0.7.0.

## Run

```sh
cargo run
```

### Linux build dependencies

Linking needs the X11 xkbcommon development files. On Debian/Ubuntu:

```sh
sudo apt install libxkbcommon-x11-dev
```

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
