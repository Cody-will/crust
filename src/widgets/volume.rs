use std::{process::Command, time::Duration};

use gpui_kit::{Context, IntoElement, Render, Window};

use crate::config::ModuleCfg;
use crate::style::box_with;

pub struct Volume {
    pub cfg: ModuleCfg,
    pub percent: u8,
    pub muted: bool,
}

impl Volume {
    pub fn new(cfg: ModuleCfg, cx: &mut Context<Self>) -> Self {
        let (percent, muted) = read_volume().unwrap_or((0, false));

        cx.spawn(async move |this, cx| {
            loop {
                let sample = read_volume();
                let _ = this.update(cx, |this, cx| {
                    let Some((percent, muted)) = sample else {
                        return;
                    };
                    if this.percent != percent || this.muted != muted {
                        this.percent = percent;
                        this.muted = muted;
                        cx.notify();
                    }
                });
                cx.background_executor()
                    .timer(Duration::from_millis(500))
                    .await;
            }
        })
        .detach();

        Self {
            cfg,
            percent,
            muted,
        }
    }
}

impl Render for Volume {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let icon = volume_icon(&self.cfg, self.percent, self.muted);
        let text = if self.muted {
            format!("{icon}   mute")
        } else {
            format!("{icon}   {}%", self.percent)
        };
        box_with(&self.cfg.class, [text])
    }
}


fn read_volume() -> Option<(u8, bool)> {
    let out = Command::new("wpctl")
        .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout);
    let muted = s.contains("MUTED");
    let frac: f32 = s.split_whitespace().nth(1)?.parse().ok()?;
    Some(((frac * 100.0).round() as u8, muted))
}


const ICON: &str = "";
const ICON_MUTE: &str = "";
const ICONS: &[&str] = &["", "", ""];


fn volume_icon(cfg: &ModuleCfg, percent: u8, muted: bool) -> String {
    if muted {
        return cfg
            .extra_str("icon_mute")
            .unwrap_or(ICON_MUTE)
            .to_string();
    }
    if let Some(toml::Value::Array(icons)) = cfg.extra.get("icons") {
        let glyphs: Vec<&str> = icons.iter().filter_map(|v| v.as_str()).collect();
        if !glyphs.is_empty() {
            let i = (percent as usize * glyphs.len() / 101).min(glyphs.len() - 1);
            return glyphs[i].to_string();
        }
    }
    cfg.extra_str("icon")
        .unwrap_or(ICON)
        .to_string()
}
