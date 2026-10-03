//! Wi-Fi pill. Auto-detects NetworkManager or iwd.
//!
//! Click runs `launch` from config, or the first GUI that exists
//! (iwgtk, nm-connection-editor, gnome-control-center wifi, …).
//!
//! `show_percent = true` appends signal. Off by default.

use std::process::Command;
use std::time::Duration;

use gpui_kit::*;
use gpui_kit::gpui::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};
use gpui_kit::gpui::WindowKind;

use crate::config::ModuleCfg;
use crate::style::apply_class;

const ICON_OFF: &str = "󰖪";
const LADDER: &[&str] = &["󰤯", "󰤟", "󰤢", "󰤥", "󰤨"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Backend {
    NetworkManager,
    Iwd,
    None,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct WifiState {
    connected: bool,
    ssid: String,
    signal: u8,
}

pub struct Wifi {
    cfg: ModuleCfg,
    backend: Backend,
    state: WifiState,
}

impl Wifi {
    pub fn new(cfg: ModuleCfg, cx: &mut Context<Self>) -> Self {
        let backend = backend_from_cfg(&cfg).unwrap_or_else(detect);
        eprintln!("crust: wifi → {backend:?}");
        let state = read_state(backend);

        cx.spawn(async move |this, cx| {
            loop {
                let _ = this.update(cx, |this, cx| {
                    let state = read_state(this.backend);
                    if this.state != state {
                        this.state = state;
                        cx.notify();
                    }
                });
                cx.background_executor()
                    .timer(Duration::from_secs(3))
                    .await;
            }
        })
        .detach();

        Self { cfg, backend, state }
    }
}

impl Render for Wifi {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let icon = wifi_icon(&self.cfg, &self.state);
        let show_pct = self
            .cfg
            .extra
            .get("show_percent")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let text = if !self.state.connected {
            icon
        } else if show_pct {
            format!("{icon}  {}%", self.state.signal)
        } else {
            icon
        };
        apply_class(div().id("wifi").cursor_pointer(), &self.cfg.class)
            .on_click(cx.listener(|this, _, _, cx| {
                let cfg = this.cfg.clone();
                let _ = cx.open_window(
                    WindowOptions {
                        titlebar: None,
                        focus: true,
                        show: true,
                        window_background: WindowBackgroundAppearance::Transparent,
                        kind: WindowKind::LayerShell(LayerShellOptions {
                            namespace: "crust-wifi".into(),
                            layer: Layer::Overlay,
                            anchor: Anchor::TOP | Anchor::LEFT | Anchor::RIGHT | Anchor::BOTTOM,
                            exclusive_zone: Some(px(-1.)),
                            margin: Some((px(0.), px(0.), px(0.), px(0.))),
                            keyboard_interactivity: KeyboardInteractivity::Exclusive,
                            ..Default::default()
                        }),
                        window_bounds: Some(WindowBounds::Windowed(Bounds {
                            origin: point(px(0.), px(0.)),
                            size: size(px(1920.), px(1080.)),
                        })),
                        ..Default::default()
                    },
                    move |_window, cx| cx.new(|cx| crate::widgets::wifi_drawer::WifiDrawer::open(cfg, cx)),
                );
            }))
            .child(text)
    }
}

fn wifi_icon(cfg: &ModuleCfg, state: &WifiState) -> String {
    if !state.connected {
        return cfg
            .extra_str("icon_off")
            .unwrap_or(ICON_OFF)
            .to_string();
    }
    if let Some(toml::Value::Array(icons)) = cfg.extra.get("icons") {
        let glyphs: Vec<&str> = icons.iter().filter_map(|v| v.as_str()).collect();
        if !glyphs.is_empty() {
            let i = (state.signal as usize * glyphs.len() / 101).min(glyphs.len() - 1);
            return glyphs[i].to_string();
        }
    }
    if let Some(icon) = cfg.extra_str("icon") {
        return icon.to_string();
    }
    let i = (state.signal as usize * LADDER.len() / 101).min(LADDER.len() - 1);
    LADDER[i].to_string()
}

fn backend_from_cfg(cfg: &ModuleCfg) -> Option<Backend> {
    Some(match cfg.extra_str("backend")? {
        "nm" | "networkmanager" | "nmcli" => Backend::NetworkManager,
        "iwd" | "iwctl" => Backend::Iwd,
        "none" => Backend::None,
        other => {
            eprintln!("crust: unknown wifi backend `{other}`");
            return None;
        }
    })
}

fn detect() -> Backend {
    if has("nmcli") && cmd_ok("nmcli", &["-t", "-f", "STATE", "general"]) {
        return Backend::NetworkManager;
    }
    if has("iwctl") {
        return Backend::Iwd;
    }
    Backend::None
}

fn read_state(backend: Backend) -> WifiState {
    match backend {
        Backend::NetworkManager => nmcli_state(),
        Backend::Iwd => iwd_state(),
        Backend::None => WifiState {
            connected: false,
            ssid: String::new(),
            signal: 0,
        },
    }
}

fn nmcli_state() -> WifiState {
    let empty = WifiState {
        connected: false,
        ssid: String::new(),
        signal: 0,
    };
    let Ok(out) = Command::new("nmcli")
        .args(["-t", "-f", "IN-USE,SSID,SIGNAL", "device", "wifi"])
        .output()
    else {
        return empty;
    };
    if !out.status.success() {
        return empty;
    }
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let mut parts = line.splitn(3, ':');
        let used = parts.next().unwrap_or("");
        let ssid = parts.next().unwrap_or("").to_string();
        let signal = parts.next().unwrap_or("0").parse().unwrap_or(0);
        if used == "*" {
            return WifiState {
                connected: true,
                ssid,
                signal,
            };
        }
    }
    empty
}

fn iwd_state() -> WifiState {
    // Best-effort: station show on the first wlan*
    let empty = WifiState {
        connected: false,
        ssid: String::new(),
        signal: 0,
    };
    let Ok(out) = Command::new("iwctl")
        .args(["station", "wlan0", "show"])
        .output()
    else {
        return empty;
    };
    if !out.status.success() {
        return empty;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut ssid = String::new();
    let mut connected = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with("Connected network") {
            ssid = line.split_whitespace().skip(2).collect::<Vec<_>>().join(" ");
            connected = !ssid.is_empty();
        }
        if line.starts_with("State") && line.contains("connected") {
            connected = true;
        }
    }
    WifiState {
        connected,
        ssid,
        signal: if connected { 80 } else { 0 },
    }
}

fn open_wifi(cfg: &ModuleCfg) {
    if let Some(cmd) = cfg.extra_str("launch") {
        let mut parts = cmd.split_whitespace();
        if let Some(bin) = parts.next() {
            let args: Vec<&str> = parts.collect();
            let _ = Command::new(bin).args(args).spawn();
        }
        return;
    }
    const CANDIDATES: &[&[&str]] = &[
        &["iwgtk"],
        &["nm-connection-editor"],
        &["nm-applet"],
        &["gnome-control-center", "wifi"],
        &["nmtui"],
    ];
    for cmd in CANDIDATES {
        if has(cmd[0]) {
            let _ = Command::new(cmd[0]).args(&cmd[1..]).spawn();
            return;
        }
    }
    eprintln!("crust: no wifi GUI found; set launch = \"…\" in config");
}

fn has(bin: &str) -> bool {
    Command::new("which")
        .arg(bin)
        .status()
        .ok()
        .is_some_and(|s| s.success())
}

fn cmd_ok(bin: &str, args: &[&str]) -> bool {
    Command::new(bin)
        .args(args)
        .output()
        .ok()
        .is_some_and(|o| o.status.success())
}
