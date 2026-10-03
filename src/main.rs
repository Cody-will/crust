//! crust: GPUI layer-shell bar.
//!
//! This file only starts the app and opens the surface. Layout lives in
//! [`app_bar::AppBar`], modules in [`registry`].

mod app_bar;
mod bar;
mod config;
mod hypr;
mod registry;
mod style;
mod widgets;

use std::path::PathBuf;

use gpui_kit::gpui::layer_shell::{Anchor, *};
use gpui_kit::gpui::WindowKind;
use gpui_kit::*;

use app_bar::AppBar;
use config::{BarPosition, Config};

fn main() {
    application().run(|cx: &mut App| {
        let path = config_path();
        let config = Config::load_or_default(path.clone());

        let bar_h = px(config.bar.height);
        let bar_w = px(crate::hypr::HyprListener::monitor_width());
        let (anchor, exclusive_edge) = anchors(config.bar.position);

        let opts = WindowOptions {
            titlebar: None,
            focus: false,
            show: true,
            is_resizable: false,
            is_minimizable: false,
            is_movable: false,
            window_decorations: Some(WindowDecorations::Client),
            window_background: WindowBackgroundAppearance::Transparent,
            app_id: Some("crust".into()),
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: point(px(0.), px(0.)),
                size: size(bar_w, bar_h),
            })),
            kind: WindowKind::LayerShell(LayerShellOptions {
                namespace: "crust".into(),
                layer: Layer::Top,
                anchor,
                exclusive_zone: Some(bar_h),
                exclusive_edge: Some(exclusive_edge),
                margin: Some((px(0.), px(0.), px(0.), px(0.))),
                keyboard_interactivity: KeyboardInteractivity::None,
            }),
            ..Default::default()
        };

        let path_for_bar = path.clone();
        cx.open_window(opts, move |window, cx| {
            window.set_exclusive_zone(bar_h);
            cx.new(|cx| AppBar::new(config, path_for_bar, cx))
        })
        .expect("layer-shell");
    });
}

fn anchors(position: BarPosition) -> (Anchor, Anchor) {
    match position {
        BarPosition::Top => (Anchor::TOP | Anchor::LEFT | Anchor::RIGHT, Anchor::TOP),
        BarPosition::Bottom => (Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT, Anchor::BOTTOM),
        BarPosition::Left => (Anchor::LEFT | Anchor::TOP | Anchor::BOTTOM, Anchor::LEFT),
        BarPosition::Right => (Anchor::RIGHT | Anchor::TOP | Anchor::BOTTOM, Anchor::RIGHT),
    }
}

fn config_path() -> PathBuf {
    if let Some(dir) = std::env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(dir).join("crust/config.toml");
    }
    std::env::var_os("HOME")
        .map(|h| PathBuf::from(h).join(".config/crust/config.toml"))
        .unwrap_or_else(|| PathBuf::from("config.toml"))
}

/// Focused monitor width in px, or 1920 if hyprctl is unavailable.
fn monitor_width() -> f32 {
    let Ok(out) = std::process::Command::new("hyprctl")
        .args(["monitors", "-j"])
        .output()
    else {
        return 1920.0;
    };
    let Ok(v) = serde_json::from_slice::<serde_json::Value>(&out.stdout) else {
        return 1920.0;
    };
    v.as_array()
        .and_then(|mons| {
            mons.iter()
                .find(|m| m["focused"].as_bool() == Some(true))
                .or_else(|| mons.first())
        })
        .and_then(|m| m["width"].as_f64())
        .unwrap_or(1920.0) as f32
}   
