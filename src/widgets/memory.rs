use std::time::Duration;

use gpui_kit::*;

use crate::config::ModuleCfg;
use crate::style::box_with;

const ICON: &str = "";

pub struct Memory {
    cfg: ModuleCfg,
    percent: u8,
    used_mib: u64,
    total_mib: u64,
}

impl Memory {
    pub fn new(cfg: ModuleCfg, cx: &mut Context<Self>) -> Self {
        let (percent, used_mib, total_mib) = read_mem().unwrap_or((0, 0, 0));
        cx.spawn(async move |this, cx| {
            loop {
                let sample = read_mem();
                let _ = this.update(cx, |this, cx| {
                    if let Some((percent, used_mib, total_mib)) = sample {
                        if this.percent != percent {
                            this.percent = percent;
                            this.used_mib = used_mib;
                            this.total_mib = total_mib;
                            cx.notify();
                        }
                    }
                });
                cx.background_executor()
                    .timer(Duration::from_secs(3))
                    .await;
            }
        })
        .detach();

        Self {
            cfg,
            percent,
            used_mib,
            total_mib,
        }
    }
}

impl Render for Memory {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let icon = self.cfg.extra_str("icon").unwrap_or(ICON);
        let text = match self.cfg.extra_str("format").unwrap_or("percent") {
            "used" => format!("{icon}  {}M", self.used_mib),
            "full" => format!("{icon}  {}/{}M", self.used_mib, self.total_mib),
            _ => format!("{icon}  {}%", self.percent),
        };
        box_with(&self.cfg.class, [text])
    }
}

fn read_mem() -> Option<(u8, u64, u64)> {
    let raw = std::fs::read_to_string("/proc/meminfo").ok()?;
    let mut total = None;
    let mut available = None;
    for line in raw.lines() {
        let mut p = line.split_whitespace();
        let key = p.next()?;
        let val: u64 = p.next()?.parse().ok()?;
        match key {
            "MemTotal:" => total = Some(val),
            "MemAvailable:" => available = Some(val),
            _ => {}
        }
    }
    let total = total?;
    let available = available?;
    let used = total.saturating_sub(available);
    let percent = ((used as f32 / total as f32) * 100.0).round() as u8;
    Some((percent, used / 1024, total / 1024))
}
