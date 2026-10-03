# Install

crust is a Hyprland bar. It will not do anything useful on GNOME, KDE, or a tty. Niri and Sway are not tested.

You need:

- Hyprland, running
- Rust stable (`rustup` is fine)
- `hyprctl` on `PATH` (it ships with Hyprland)
- PipeWire and `wpctl` if you want the volume module to show a number
- NetworkManager and `nmcli` if you want the wifi pill and drawer
- One of swaync, dunst, or mako if you want the notification count

Battery, cpu, and memory read `/sys` and `/proc`. They do not need an extra package. A desktop with no battery just skips that pill.

## From a git checkout

```sh
git clone https://github.com/Cody-will/crust.git
cd crust
cargo build --release
install -Dm755 target/release/crust ~/.local/bin/crust
```

Swap the GitHub path for wherever you actually host it. `~/.local/bin` needs to be on `PATH`. A distro package does not exist yet.

## First run

```sh
crust
```

If `~/.config/crust/config.toml` is missing or empty, crust writes the built-in mocha config and opens the bar. If you already have a file, it uses that and does not overwrite it.

Hyprland:

```ini
exec-once = crust
```

Remove any `exec-once = waybar` line. Both want the same edge.

Reload Hyprland or log out and back in. `hyprctl layers` should show a `crust` surface on `top` with a height close to what you set in `[bar].height`.

## Config path

`$XDG_CONFIG_HOME/crust/config.toml` if that variable is set, otherwise `~/.config/crust/config.toml`.

Editing the file is enough. crust checks the mtime about three times a second and respawns modules. It does not watch the directory with inotify, on purpose. A blocking watcher froze the bar.

## Themes

`theme = "mocha"`, `"latte"`, `"light"`, `"zinc"`, or `"dark"`. Unknown names fall back to mocha.

To see the theme actually change, the classes have to use `bg-surface`, `text-text`, `bg-accent`, or `text-muted`. `bg-zinc-900` is a fixed color. It will not move when you switch themes.

## If the bar reserves space but paints nothing

The exclusive zone is set even when the first frame is empty. Check the terminal you launched from. A parse error in the TOML is printed there, and crust falls back to the built-in theme only when the file is empty, invalid, or has no modules. A valid file with a broken class still opens. Unknown classes are logged and skipped.

## Uninstall

Remove the binary and, if you want the config gone too:

```sh
rm ~/.local/bin/crust
rm -rf ~/.config/crust
```

Take the `exec-once` line out of `hyprland.conf`.
