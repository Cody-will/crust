# crust

A status bar for Hyprland, written in Rust. It sits on the edge of the screen, which is also where the crust of a pizza sits. That is the whole joke. It does not get funnier.

crust is a layer-shell bar. You describe modules in a TOML file and style them with short class strings that look like Tailwind (`px-2`, `rounded-md`, `bg-surface/90`). There is no webview and no GTK. The first time you run it, if you do not already have a config, it writes one and opens a mocha bar.

![mocha bar on a dark desktop](docs/images/mocha.jpg)

The pictures in this repo are concept renders, not screenshots of a release build. Your bar will look like whatever you put in the config. The default is closer to the dark one than the floating pill one.

![logo](docs/images/logo.jpg)

## What you get

Workspaces (click to focus), focused window title, clock, cpu, memory, volume, wifi, battery, notifications, a static label, and `exec` for anything else. Wifi can open a drawer that lists networks and connects through NetworkManager. That drawer is newer than the bar. Treat it as useful, not finished.

Themes are `mocha` (default), `latte` / `light`, and `zinc` / `dark`. `bg-surface`, `text-text`, `bg-accent`, and `text-muted` follow the theme. Normal Tailwind-ish colors (`bg-zinc-900`, `text-blue-400`) do not.

## Running it

Hyprland, a Rust toolchain, and the usual suspects (`hyprctl`, `wpctl` for volume, `nmcli` if you want wifi). Notifications talk to swaync, dunst, or mako, whichever is actually running.

```sh
cargo run --release
```

Config lands at `$XDG_CONFIG_HOME/crust/config.toml`, or `~/.config/crust/config.toml`. An empty file is replaced with the built-in theme. A file you already wrote is left alone.

In `hyprland.conf`:

```ini
exec-once = crust
```

Take waybar out of `exec-once` if it is still there. Two bars will both reserve the top edge and one of them will look like it lost.

Install details are in [INSTALL.md](INSTALL.md). If you want to change the code, start at [CONTRIBUTING.md](CONTRIBUTING.md). A fuller config is in [examples/config.toml](examples/config.toml).

## Config, short version

`[bar]` is the strip. `[bar.start]`, `[bar.center]`, and `[bar.end]` are the three columns. `[[start]]`, `[[center]]`, and `[[end]]` are the modules, top to bottom in the file, left to right on the bar. The single-bracket table and the double-bracket array are not the same key.

```toml
theme = "mocha"

[bar]
position = "top"
height = 36
class = "w-full h-full items-center bg-surface/90 text-text"

[bar.start]
class = "flex items-center gap-2"
container_class = "justify-start px-2"

[[start]]
type = "workspaces"
class = "flex gap-1 text-text"

[start.parts]
item = "px-2 rounded text-muted"
active = "px-2 rounded bg-accent text-surface"

[[center]]
type = "clock"
format = "%H:%M"
class = "px-3 text-sm text-text"
```

`container_class` is the column. `class` is the pack of modules inside it. Put `justify-center` on the container if you want the clock actually centered. Putting `w-full` on the pack fights that.

`exec` is the escape hatch:

```toml
[[end]]
type = "exec"
command = "date +%H:%M"
interval = 30
on_click = "kitty"
class = "px-2 text-sm text-text"
```

## Not in this version

Tray, media players, per-monitor bars, and a plugin loader. Position is read at startup. Changing `position` in the file does not move the surface until you restart. The wifi drawer blocks while `nmcli` connects. That is a known rough edge.
