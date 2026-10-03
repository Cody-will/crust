use std::time::Duration;

use gpui_kit::*;

use crate::config::ModuleCfg;
use crate::style::box_with;

const ICON: &str = "";

pub struct Cpu {
    cfg: ModuleCfg,
    percent: u8,
    prev: Option<(u64, u64)>,
}

impl Cpu {
    pub fn new(cfg: ModuleCfg, cx: &mut Context<Self>) -> Self {
        cx.spawn(async move |this, cx| {
            loop {
                let sample = read_cpu();
                let _ = this.update(cx, |this, cx| {
                    if let Some((idle, total)) = sample {
                        if let Some((p_idle, p_total)) = this.prev {
                            let di = idle.saturating_sub(p_idle);
                            let dt = total.saturating_sub(p_total);
                            if dt > 0 {
                                let busy = 1.0 - (di as f32 / dt as f32);
                                this.percent = (busy * 100.0).round().clamp(0.0, 100.0) as u8;
                                cx.notify();
                            }
                        }
                        this.prev = Some((idle, total));
                    }
                });
                cx.background_executor()
                    .timer(Duration::from_secs(2))
                    .await;
            }
        })
        .detach();

        Self {
            cfg,
            percent: 0,
            prev: read_cpu(),
        }
    }
}

impl Render for Cpu {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let icon = self.cfg.extra_str("icon").unwrap_or(ICON);
        box_with(
            &self.cfg.class,
            [format!("{icon}  {}%", self.percent)],
        )
    }
}

/// First line of /proc/stat: cpu user nice system idle iowait irq softirq steal …
fn read_cpu() -> Option<(u64, u64)> {
    let line = std::fs::read_to_string("/proc/stat").ok()?;
    let line = line.lines().next()?;
    let mut n = line.split_whitespace().skip(1).filter_map(|s| s.parse::<u64>().ok());
    let user = n.next()?;
    let nice = n.next()?;
    let system = n.next()?;
    let idle = n.next()?;
    let iowait = n.next().unwrap_or(0);
    let irq = n.next().unwrap_or(0);
    let softirq = n.next().unwrap_or(0);
    let steal = n.next().unwrap_or(0);
    let idle_all = idle + iowait;
    let total = user + nice + system + idle_all + irq + softirq + steal;
    Some((idle_all, total))
}
