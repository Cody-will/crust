# Contributing

PRs are welcome. Small ones that match the rest of the code are easier to take.

crust is a Hyprland bar. A change that needs GNOME, or that shells out to a browser, is probably the wrong project.

## Layout of the code

- `main.rs` opens the layer-shell surface and nothing else.
- `app_bar.rs` owns the config, the hot reload, and the module lists.
- `bar.rs` is the three columns. It does not know what a clock is.
- `registry.rs` maps `type = "clock"` to a constructor. A new built-in is a variant there plus a file under `widgets/`.
- `style.rs` is the class parser. `apply_class` is what widgets call. `set_theme` is what makes `bg-surface` work.
- `config.rs` is the TOML shape and the built-in theme.
- `hypr.rs` is the socket listener and the workspace snapshot.

Do not teach a widget how to parse TOML. It gets a `ModuleCfg`. Do not teach `Bar` about workspaces.

## Adding a module

1. New file in `src/widgets/`. If it changes over time, it is a view: store state, `cx.spawn` the poll, `cx.notify()` when the value changes.
2. Add the name to `BuiltinModule` and `spawn_module`.
3. Style only through `cfg.class` and `cfg.parts`, passed to `apply_class`. Hardcoded colors will ignore the theme.
4. Clicks that run a program should `spawn`, not `status`. Waiting on `hyprctl` or a daemon freezes the bar.

`exec` already covers "run this and print a line." A built-in is worth it when you need icons, click behavior, or structured state.

## Class strings

If you add a token, put it in `apply_token` or `apply_dynamic`, and mention it in the module docs at the top of `style.rs`. `border-2` is a width. Anything that starts with `border-` and is not a width is a color. That split exists because `apply_class_with` used to send `border-2` down the color path and log it as unknown.

Theme names (`surface`, `text`, `accent`, `muted`) come from `set_theme`. Do not thread a color map through every widget unless you have a reason.

## Before you open a PR

```sh
cargo build
cargo clippy
```

Run it on Hyprland. Click the thing you changed. If the bar hitches, the click is blocking.

Do not reformat unrelated files. Do not bump the gpui-kit pin unless the build is actually broken.

## Bugs

A useful report has the crust commit, the Hyprland version, the module config that failed, and the line crust printed. `hyprctl layers` helps when the bar reserves space and does not paint.
