use gpui_kit::{*, SharedString};
use crate::config::ModuleCfg;
use std::time::Duration;
use crate::style::box_with;



pub struct Clock {
    pub cfg: ModuleCfg,
    pub time: SharedString,
}

impl Clock {
    pub fn new(cfg: ModuleCfg, cx: &mut Context<Self>) -> Self {
        let format = cfg
            .extra_str("format")
            .unwrap_or("%H:%M")
            .to_string();

        cx.spawn(async move |this, cx| {
            loop {
                let stamp = chrono::Local::now().format(&format).to_string();
                if this
                    .update(cx, |this, cx| {
                        this.time = stamp.into();
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
                cx.background_executor()
                    .timer(Duration::from_secs(1))
                    .await;
            }
        })
        .detach();

        Self {
            cfg,
            time: "--:--".into(),
        }
    }
}

impl Render for Clock {
    fn render(&mut self, _:&mut Window, _:&mut Context<Self>) -> impl IntoElement {
        box_with(&self.cfg.class, [self.time.clone()])
    }
}
