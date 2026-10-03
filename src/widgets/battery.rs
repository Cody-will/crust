use std::path::PathBuf;
use std::time::Duration;

use gpui_kit::*;

use crate::config::ModuleCfg;
use crate::style::box_with;

const ICON_CHARGING: &str = "󰂄";
const ICON_FULL: &str = "󰁹";
const LADDER: &[&str] = &["󰂎", "󰁺", "󰁻", "󰁼", "󰁽", "󰁾", "󰁿", "󰂀", "󰂁", "󰂂", "󰁹"];

pub struct Battery {
    cfg: ModuleCfg,
    percent: u8,
    charging: bool,
    present: bool,
}

impl Battery {
    pub fn new(cfg: ModuleCfg, cx: &mut Context<Self>) -> Self {
        let (percent, charging, present) = read_battery().unwrap_or((0, false, false));
        cx.spawn(async move |this, cx| {
            loop {
                let sample = read_battery();
                let _ = this.update(cx, |this, cx| {
                    if let Some((percent, charging, present)) = sample {
                        if this.percent != percent
                            || this.charging != charging
                            || this.present != present
                        {
                            this.percent = percent;
                            this.charging = charging;
                            this.present = present;
                            cx.notify();
                        }
                    } else if this.present {
                        this.present = false;
                        cx.notify();
                    }
                });
                cx.background_executor()
                    .timer(Duration::from_secs(10))
                    .await;
            }
        })
        .detach();

        Self {
            cfg,
            percent,
            charging,
            present,
        }
    }
}

impl Render for Battery {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        if !self.present {
            return div(); 
        }
        let icon = battery_icon(&self.cfg, self.percent, self.charging);
        box_with(
            &self.cfg.class,
            [format!("{icon} {}%", self.percent)],
        )
    }
}

fn battery_icon(cfg: &ModuleCfg, percent: u8, charging: bool) -> String {
    if charging {
        return cfg
            .extra_str("icon_charging")
            .unwrap_or(ICON_CHARGING)
            .to_string();
    }
    if let Some(toml::Value::Array(icons)) = cfg.extra.get("icons") {
        let glyphs: Vec<&str> = icons.iter().filter_map(|v| v.as_str()).collect();
        if !glyphs.is_empty() {
            let i = (percent as usize * glyphs.len() / 101).min(glyphs.len() - 1);
            return glyphs[i].to_string();
        }
    }
    if percent >= 95 {
        return cfg.extra_str("icon").unwrap_or(ICON_FULL).to_string();
    }
    let i = (percent as usize * LADDER.len() / 101).min(LADDER.len() - 1);
    LADDER[i].to_string()
}

fn read_battery() -> Option<(u8, bool, bool)> {
    let dir = battery_dir()?;
    let cap = std::fs::read_to_string(dir.join("capacity")).ok()?;
    let percent = cap.trim().parse::<u8>().ok()?;
    let status = std::fs::read_to_string(dir.join("status")).unwrap_or_default();
    let charging = status.trim().eq_ignore_ascii_case("charging")
        || status.trim().eq_ignore_ascii_case("full");
    Some((percent, charging, true))
}

fn battery_dir() -> Option<PathBuf> {
    let root = PathBuf::from("/sys/class/power_supply");
    for ent in std::fs::read_dir(&root).ok()? {
        let ent = ent.ok()?;
        let name = ent.file_name();
        let name = name.to_string_lossy();
        if name.starts_with("BAT") {
            return Some(ent.path());
        }
    }
    None
}
