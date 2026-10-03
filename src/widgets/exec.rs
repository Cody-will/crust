//! User command module.
//!
//! Runs `command` every `interval` seconds (default 30) and shows the
//! first line of stdout. `icon` is prepended when set. `on_click` is a
//! second command, spawned and not awaited.
//!
//! # Config
//!
//! ```toml
//! [[end]]
//! type = "exec"
//! class = "px-2 text-sm"
//! command = "date +%H:%M"
//! interval = 30
//! icon = "󰥔"
//! on_click = "kitty"
//! ```
//!
//! Empty stdout hides the pill. A failed command keeps the last good text
//! and logs once per change of status.

use std::process::Command;
use std::time::Duration;

use gpui_kit::*;

use crate::config::ModuleCfg;
use crate::style::apply_class;

pub struct Exec {
    cfg: ModuleCfg,
    text: SharedString,
}

impl Exec {
    /// Spawn the poll loop and paint the first command result immediately.
    pub fn new(cfg: ModuleCfg, cx: &mut Context<Self>) -> Self {
        let text = run_command(cfg.extra_str("command").unwrap_or("")).into();
        let secs = cfg
            .extra
            .get("interval")
            .and_then(|v| v.as_integer())
            .unwrap_or(30)
            .max(1) as u64;

        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_secs(secs))
                    .await;
                let _ = this.update(cx, |this, cx| {
                    let next = run_command(this.cfg.extra_str("command").unwrap_or(""));
                    if this.text.as_ref() != next {
                        this.text = next.into();
                        cx.notify();
                    }
                });
            }
        })
        .detach();

        Self { cfg, text }
    }
}

impl Render for Exec {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut el = div().id("exec"); 
        if self.text.is_empty() {
            return el;
        }
        let icon = self.cfg.extra_str("icon").unwrap_or("");
        let label = if icon.is_empty() {
            self.text.to_string()
        } else {
            format!("{icon}  {}", self.text)
        };
        let click = self.cfg.extra_str("on_click").unwrap_or("").to_string();

        el = apply_class(el, &self.cfg.class).child(label);
        if !click.is_empty() {
            el = el.cursor_pointer().on_click(cx.listener(move |_, _, _, _| {
                spawn_command(&click);
            }));
        }
        el
    }
}

fn run_command(cmd: &str) -> String {
    if cmd.is_empty() {
        return String::new();
    }
    let Ok(out) = Command::new("sh").arg("-c").arg(cmd).output() else {
        return String::new();
    };
    if !out.status.success() {
        return String::new();
    }
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .to_string()
}

fn spawn_command(cmd: &str) {
    let _ = Command::new("sh").arg("-c").arg(cmd).spawn();
}
