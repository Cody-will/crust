//! Notification count pill. Auto-detects swaync, dunst, or mako.
//!
//! Click toggles the daemon's panel (swaync) or a best-effort action
//! (dunst history-pop / mako restore). Override with `daemon = "swaync"`
//! in the module TOML.

use std::process::Command;
use std::time::Duration;

use gpui_kit::*;

use crate::config::ModuleCfg;
use crate::style::apply_class;

const ICON: &str = "󰂚";
const ICON_NONE: &str = "󰂛";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Daemon {
    Swaync,
    Dunst,
    Mako,
    None,
}

pub struct Notifications {
    cfg: ModuleCfg,
    daemon: Daemon,
    count: u32,
}

impl Notifications {
    pub fn new(cfg: ModuleCfg, cx: &mut Context<Self>) -> Self {
        let daemon = daemon_from_cfg(&cfg).unwrap_or_else(detect);
        eprintln!("crust: notifications → {daemon:?}");
        let count = read_count(daemon);

        cx.spawn(async move |this, cx| {
            loop {
                let _ = this.update(cx, |this, cx| {
                    let count = read_count(this.daemon);
                    if this.count != count {
                        this.count = count;
                        cx.notify();
                    }
                });
                cx.background_executor()
                    .timer(Duration::from_secs(1))
                    .await;
            }
        })
        .detach();

        Self { cfg, daemon, count }
    }
}

impl Render for Notifications {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let icon = if self.count == 0 {
            self.cfg.extra_str("icon_none").unwrap_or(ICON_NONE)
        } else {
            self.cfg.extra_str("icon").unwrap_or(ICON)
        };
        let text = if self.count == 0 {
            icon.to_string()
        } else {
            format!("{icon} {}", self.count)
        };

        apply_class(div().id("notifications").cursor_pointer(), &self.cfg.class)
            .on_click(cx.listener(|this, _, _, _| {
                toggle_panel(this.daemon);
            }))
            .child(text)
    }
}

fn daemon_from_cfg(cfg: &ModuleCfg) -> Option<Daemon> {
    Some(match cfg.extra_str("daemon")? {
        "swaync" => Daemon::Swaync,
        "dunst" => Daemon::Dunst,
        "mako" => Daemon::Mako,
        "none" => Daemon::None,
        other => {
            eprintln!("crust: unknown notification daemon `{other}`");
            return None;
        }
    })
}

fn detect() -> Daemon {
    if alive("swaync") && has("swaync-client") {
        return Daemon::Swaync;
    }
    if alive("dunst") && has("dunstctl") {
        return Daemon::Dunst;
    }
    if alive("mako") && has("makoctl") {
        return Daemon::Mako;
    }
    if cmd_ok("swaync-client", &["-c"]) {
        return Daemon::Swaync;
    }
    if cmd_ok("dunstctl", &["is-paused"]) {
        return Daemon::Dunst;
    }
    if cmd_ok("makoctl", &["list"]) {
        return Daemon::Mako;
    }
    Daemon::None
}

fn has(bin: &str) -> bool {
    Command::new("which")
        .arg(bin)
        .status()
        .ok()
        .is_some_and(|s| s.success())
}

fn alive(name: &str) -> bool {
    Command::new("pidof")
        .arg(name)
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

fn cmd_u32(bin: &str, args: &[&str]) -> Option<u32> {
    let out = Command::new(bin).args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8_lossy(&out.stdout).trim().parse().ok()
}

fn read_count(daemon: Daemon) -> u32 {
    match daemon {
        Daemon::Swaync => cmd_u32("swaync-client", &["-c"]).unwrap_or(0),
        Daemon::Dunst => cmd_u32("dunstctl", &["count", "waiting"]).unwrap_or(0),
        Daemon::Mako => mako_count(),
        Daemon::None => 0,
    }
}

fn mako_count() -> u32 {
    let Ok(out) = Command::new("makoctl").arg("list").output() else {
        return 0;
    };
    if !out.status.success() {
        return 0;
    }
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| {
            let t = l.trim();
            t.starts_with("Notification") || t.contains("app-name")
        })
        .count() as u32
}

fn toggle_panel(daemon: Daemon) {
    match daemon {
        Daemon::Swaync => {
            let _ = Command::new("swaync-client").arg("-t").spawn();
        }
        Daemon::Dunst => {
            let _ = Command::new("dunstctl").arg("history-pop").spawn();
        }
        Daemon::Mako => {
            let _ = Command::new("makoctl").arg("restore").spawn();
        }
        Daemon::None => {
            eprintln!("crust: no notification daemon to open");
        }
    }
}
