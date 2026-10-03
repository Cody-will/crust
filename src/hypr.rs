//! Hyprland snapshot and event ping.
//!
//! One [`HyprListener`] per widget is fine. The thread only sends `()`.
//! [`HyprListener::snapshot`] is the read. Clicks must [`std::process::Command::spawn`],
//! not `status`, so the UI thread does not wait on hyprctl.

use std::sync::mpsc::{self, Receiver};
use std::thread;

use hyprland::data::{Client, Workspace, Workspaces, Monitors};
use hyprland::event_listener::EventListener;
use hyprland::prelude::*;

#[derive(Clone, Debug)]
pub struct HyprSnapshot {
    pub active_workspace: i32,
    pub workspace_ids: Vec<i32>,
    pub window_title: String,
    pub window_class: String,
}

impl HyprSnapshot {
    pub fn into_items(self) -> Vec<crate::widgets::workspaces::Workspace> {
        self.workspace_ids
            .iter()
            .map(|id| crate::widgets::workspaces::Workspace {
                id: *id,
                name: id.to_string().into(),
                active: *id == self.active_workspace,
            })
            .collect()
    }
}

pub struct HyprListener {
    rx: Receiver<()>,
}

impl HyprListener {
    /// Background thread on socket2. Ping on workspace or active-window change.
    pub fn start() -> Self {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let mut listener = EventListener::new();
            let ping = move || {
                let _ = tx.send(());
            };

            listener.add_workspace_changed_handler({
                let ping = ping.clone();
                move |_| ping()
            });
            listener.add_workspace_added_handler({
                let ping = ping.clone();
                move |_| ping()
            });
            listener.add_workspace_deleted_handler({
                let ping = ping.clone();
                move |_| ping()
            });
            listener.add_active_window_changed_handler({
                let ping = ping.clone();
                move |_| ping()
            });

            if let Err(e) = listener.start_listener() {
                eprintln!("crust: hyprland listener stopped: {e}");
            }
        });
        Self { rx }
    }

    /// True if at least one event is waiting. Does not block.
    pub fn try_wait(&self) -> bool {
        self.rx.try_recv().is_ok()
    }

    /// Focus a workspace. Does not wait for hyprctl.
    pub fn dispatch_workspace(id: i32) {
        let expr = format!(r#"hl.dsp.focus({{ workspace = "{id}" }})"#);
        let _ = std::process::Command::new("hyprctl")
            .args(["dispatch", &expr])
            .spawn();
    }

    /// Current workspaces, active id, and focused window.
    pub fn snapshot() -> HyprSnapshot {
        let active = Workspace::get_active().ok();
        let mut workspace_ids: Vec<i32> = Workspaces::get()
            .map(|list| list.into_iter().map(|w| w.id).collect())
            .unwrap_or_default();
        workspace_ids.sort();
        let win = Client::get_active().ok().flatten();

        HyprSnapshot {
            active_workspace: active.as_ref().map(|w| w.id).unwrap_or(1),
            workspace_ids,
            window_title: win.as_ref().map(|w| w.title.clone()).unwrap_or_default(),
            window_class: win.as_ref().map(|w| w.class.clone()).unwrap_or_default(),
        }
    }

    /// Focused monitor width in px. 1920 if the socket read fails.
    pub fn monitor_width() -> f32 {
        Monitors::get()
            .ok()
            .and_then(|list| {
                let all: Vec<_> = list.into_iter().collect();
                all.iter()
                    .find(|m| m.focused)
                    .or_else(|| all.first())
                    .map(|m| m.width as f32)
            })
            .unwrap_or(1920.0)
    }
}



