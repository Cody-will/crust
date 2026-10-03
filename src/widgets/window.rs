use std::time::Duration;

use gpui_kit::*;

use crate::config::ModuleCfg;
use crate::hypr::HyprListener;
use crate::style::apply_class;

pub struct WindowTitle {
    cfg: ModuleCfg,
    title: SharedString,
    class_name: SharedString,
}

impl WindowTitle {
    pub fn new(cfg: ModuleCfg, cx: &mut Context<Self>) -> Self {
        let snap = HyprListener::snapshot();
        let listener = HyprListener::start();

        cx.spawn(async move |this, cx| {
            loop {
                if listener.try_wait() {
                    let snap = HyprListener::snapshot();
                    let _ = this.update(cx, |this, cx| {
                        this.title = snap.window_title.into();
                        this.class_name = snap.window_class.into();
                        cx.notify();
                    });
                }
                cx.background_executor()
                    .timer(Duration::from_millis(50))
                    .await;
            }
        })
        .detach();

        Self {
            cfg,
            title: snap.window_title.into(),
            class_name: snap.window_class.into(),
        }
    }
}


impl Render for WindowTitle {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        if self.title.is_empty() {
            return div();
        }

        let max = self
            .cfg
            .extra
            .get("max_length")
            .and_then(|v| v.as_integer())
            .unwrap_or(40) as usize;
        let text: String = self.title.chars().take(max).collect();
        apply_class(div(), &self.cfg.class).child(text)
    }
}







