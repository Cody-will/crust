use std::time::Duration;

use gpui_kit::*;

use crate::config::ModuleCfg;
use crate::hypr::HyprListener;
use crate::style::apply_class;

#[derive(Debug, Clone, PartialEq)]
pub struct Workspace {
    pub id: i32,
    pub name: SharedString,
    pub active: bool,
}

#[derive(Debug)]
pub struct WorkSpaces {
    pub cfg: ModuleCfg,
    pub items: Vec<Workspace>,
}

impl WorkSpaces {
    pub fn new(cfg: ModuleCfg, cx: &mut Context<Self>) -> Self {
        let items = HyprListener::snapshot().into_items();
        let listener = HyprListener::start();

        cx.spawn(async move |this, cx| {
            loop {
                if listener.try_wait() {
                    let snap = HyprListener::snapshot();
                    let _ = this.update(cx, |this, cx| {
                        this.items = snap.into_items();
                        cx.notify();
                    });
                }
                cx.background_executor()
                    .timer(Duration::from_millis(50))
                    .await;
            }
        })
        .detach();

        Self { cfg, items }
    }
}

impl Render for WorkSpaces {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let children: Vec<_> = self
            .items
            .iter()
            .map(|ws| {
                let class = if ws.active {
                    self.cfg
                        .parts
                        .get("active")
                        .map(String::as_str)
                        .unwrap_or("px-2 rounded")
                } else {
                    self.cfg
                        .parts
                        .get("item")
                        .map(String::as_str)
                        .unwrap_or("px-2")
                };
                let id = ws.id;
                apply_class(div().id(SharedString::from(format!("ws-{id}"))), class)
                    .cursor_pointer()
                    .on_click(cx.listener(move |_, _, _, _| {
                        HyprListener::dispatch_workspace(id);
                    }))
                    .child(ws.name.clone())
            })
            .collect();

        apply_class(div(), &self.cfg.class).children(children)
    }
}
