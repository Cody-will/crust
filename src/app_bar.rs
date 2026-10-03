//! Root view for one crust window.
//!
//! Owns the loaded [`Config`], the config file path, and the module
//! instances for the three slots. [`Bar`] only lays those instances out.
//!
//! On construct and reload this calls [`crate::style::set_theme`] so every
//! widget's [`crate::style::apply_class`] can resolve `bg-surface` and friends.
//!
//! Slot chrome is two layers:
//! - `[bar.start].container_class` → `#start-container` (`flex-1` column)
//! - `[bar.start].class` → `#start` (packed modules inside that column)
//!
//! # Hot reload
//!
//! Polls mtime every 300ms. Do not `recv()` on this task.

use std::path::PathBuf;
use std::time::Duration;

use gpui_kit::{
    Context, IntoElement, ParentElement, Render, Styled, Window, div, px,
};

use crate::{
    bar::Bar,
    config::Config,
    registry::{spawn_all, SlotItem},
    style,
};

/// Window root. One per layer-shell surface.
pub struct AppBar {
    /// Last successfully loaded config. Read from [`Render`] for classes/height.
    pub config: Config,
    /// File [`watch_config`] stats and [`reload`] parses.
    pub path: PathBuf,
    pub start: Vec<SlotItem>,
    pub center: Vec<SlotItem>,
    pub end: Vec<SlotItem>,
}

impl AppBar {
    /// Spawn modules from `config` and watch `path` for changes.
    pub fn new(config: Config, path: PathBuf, cx: &mut Context<Self>) -> Self {
        let config = config.with_theme_colors();
        style::set_theme(config.colors.clone());
        Self::watch_config(path.clone(), cx);

        let start = spawn_all(&config.start, cx);
        let center = spawn_all(&config.center, cx);
        let end = spawn_all(&config.end, cx);
        Self {
            config,
            path,
            start,
            center,
            end,
        }
    }

    /// Poll `path`'s mtime on the background executor.
    fn watch_config(path: PathBuf, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            let mut last = std::fs::metadata(&path)
                .ok()
                .and_then(|m| m.modified().ok());
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(300))
                    .await;
                let now = std::fs::metadata(&path)
                    .ok()
                    .and_then(|m| m.modified().ok());
                if now != last {
                    last = now;
                    let _ = this.update(cx, |bar, cx| bar.reload(cx));
                }
            }
        })
        .detach();
    }

    /// Re-read `self.path`. Unusable files become the built-in theme.
    fn reload(&mut self, cx: &mut Context<Self>) {
        self.config = Config::load_or_default(&self.path).with_theme_colors();
        style::set_theme(self.config.colors.clone());
        self.start = spawn_all(&self.config.start, cx);
        self.center = spawn_all(&self.config.center, cx);
        self.end = spawn_all(&self.config.end, cx);
        cx.notify();
        eprintln!("crust: reloaded {}", self.path.display());
    }
}

impl Render for AppBar {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        window.set_exclusive_zone(px(self.config.bar.height));

        let bar_class = if self.config.bar.class.is_empty() {
            "w-full h-full items-center bg-surface/90 text-text".to_string()
        } else {
            self.config.bar.class.clone()
        };

        let mut bar = Bar::new(&bar_class)
            .colors(self.config.colors.clone())
            .start_class(&self.config.bar.start.class)
            .center_class(&self.config.bar.center.class)
            .end_class(&self.config.bar.end.class)
            .start_container_class(&self.config.bar.start.container_class)
            .center_container_class(&self.config.bar.center.container_class)
            .end_container_class(&self.config.bar.end.container_class);

        for item in &self.start {
            bar = bar.start(item.to_element());
        }
        for item in &self.center {
            bar = bar.center(item.to_element());
        }
        for item in &self.end {
            bar = bar.end(item.to_element());
        }

        div().size_full().child(bar)
    }
}
