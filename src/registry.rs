//! Module catalog: config `type` strings → [`SlotItem`]s.
//!
//! [`Bar`](crate::bar::Bar) does not know what a clock is. It only receives
//! children. This file is the map from `type = "clock"` to a constructor.
//!
//! # Adding a built-in
//!
//! 1. Implement the widget under `crate::widgets`.
//! 2. Add a [`BuiltinModule`] variant and a [`BuiltinModule::parse`] arm.
//! 3. Handle it in [`spawn_module`] (view) and/or [`render_leaf`] (leaf).
//!
//! # Examples
//!
//! ```ignore
//! let items = spawn_all(&config.center, cx);
//! // each SlotItem becomes a child in AppBar::render via to_element()
//! ```

use gpui_kit::{AnyElement, AnyView, Context, AppContext, IntoElement, div};

use crate::{
    app_bar::AppBar,
    config::ModuleCfg,
    style::box_with,
    widgets::{
        clock::Clock,
        cpu::Cpu,
        memory::Memory,
        volume::Volume,
        workspaces::WorkSpaces,
        battery::Battery,
        notifications::Notifications,
        wifi::Wifi,
        window::WindowTitle,
        exec::Exec,
    },
};

/// Something a slot can hold across frames.
///
/// Views keep the entity alive (timers/IPC). Leaves store config and are
/// rebuilt every paint — [`AnyElement`] is not [`Clone`].
///
/// # Examples
///
/// ```ignore
/// match spawn_module(cfg, cx)? {
///     SlotItem::View(v) => { /* entity lives on AppBar */ }
///     SlotItem::Leaf(cfg) => { /* painted via render_leaf */ }
/// }
/// ```
#[derive(Clone)]
pub enum SlotItem {
    /// A GPUI view handle (`Clock`, `Workspaces`, …).
    View(AnyView),
    /// Config for a stateless widget, painted in [`render_leaf`].
    Leaf(ModuleCfg),
}

impl SlotItem {
    /// Build this frame’s element tree. Call only from `Render`.
    pub fn to_element(&self) -> AnyElement {
        match self {
            Self::View(v) => v.clone().into_any_element(),
            Self::Leaf(cfg) => render_leaf(cfg),
        }
    }
}

/// Paint a leaf module from its config.
///
/// Unknown kinds log and return an empty `div()`.
pub fn render_leaf(cfg: &ModuleCfg) -> AnyElement {
    match cfg.kind.as_str() {
        "volume" => volume_pill(cfg).into_any_element(),
        "label" => {
            let text = cfg.extra_str("text").unwrap_or("");
            box_with(&cfg.class, [text.to_string()]).into_any_element()
        }
        other => {
            eprintln!("crust: `{other}` is not a leaf");
            div().into_any_element()
        }
    }
}

fn volume_pill(cfg: &ModuleCfg) -> impl IntoElement {
    box_with(&cfg.class, ["vol".to_string()])
}

/// Closed set of module names this binary ships.
///
/// Config still uses a [`String`] so unknown `type`s parse; [`parse`] maps
/// them here or yields [`None`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinModule {
    Clock,
    Workspaces,
    Volume,
    Cpu,
    Memory,
    Battery,
    Notifications,
    Wifi,
    Label,
    WindowTitle,
    Exec,
}

impl BuiltinModule {
    /// Map a TOML `type` field onto a built-in.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// assert_eq!(BuiltinModule::parse("clock"), Some(BuiltinModule::Clock));
    /// assert_eq!(BuiltinModule::parse("tray"), None);
    /// ```
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "clock" => Some(Self::Clock),
            "workspaces" | "workspace" => Some(Self::Workspaces),
            "volume" => Some(Self::Volume),
            "cpu" => Some(Self::Cpu),
            "memory" => Some(Self::Memory),
            "battery" => Some(Self::Battery),
            "notifications" => Some(Self::Notifications),
            "network" | "wifi" => Some(Self::Wifi),
            "window" | "window_title" => Some(Self::WindowTitle),
            "exec" => Some(Self::Exec),
            "label" => Some(Self::Label),
            _ => None,
        }
    }
}

/// Construct one module. [`None`] if the type is unknown or not wired yet.
///
/// Views use `cx.new`. Leaves clone the [`ModuleCfg`] only.
pub fn spawn_module(cfg: &ModuleCfg, cx: &mut Context<AppBar>) -> Option<SlotItem> {
    match BuiltinModule::parse(&cfg.kind) {
        Some(BuiltinModule::Clock) => Some(SlotItem::View(cx.new(|cx| Clock::new(cfg.clone(), cx)).into())),
        Some(BuiltinModule::Volume) => Some(SlotItem::View(cx.new(|cx| Volume::new(cfg.clone(), cx)).into())),
        Some(BuiltinModule::Workspaces) => Some(SlotItem::View(cx.new(|cx| WorkSpaces::new(cfg.clone(), cx)).into())),
        Some(BuiltinModule::Cpu) => Some(SlotItem::View(cx.new(|cx| Cpu::new(cfg.clone(), cx)).into())),
        Some(BuiltinModule::Memory) => Some(SlotItem::View(cx.new(|cx| Memory::new(cfg.clone(), cx)).into())),
        Some(BuiltinModule::Battery) => Some(SlotItem::View(cx.new(|cx| Battery::new(cfg.clone(), cx)).into())),
        Some(BuiltinModule::Notifications) => Some(SlotItem::View(cx.new(|cx| Notifications::new(cfg.clone(), cx)).into())),
        Some(BuiltinModule::Wifi) => Some(SlotItem::View(cx.new(|cx| Wifi::new(cfg.clone(), cx)).into())),
        Some(BuiltinModule::WindowTitle) => Some(SlotItem::View(cx.new(|cx| WindowTitle::new(cfg.clone(), cx)).into())),
        Some(BuiltinModule::Exec)  => Some(SlotItem::View(cx.new(|cx| Exec::new(cfg.clone(), cx)).into())),
        Some(BuiltinModule::Label) => {
            Some(SlotItem::Leaf(cfg.clone()))
        }
        None => {
            eprintln!("crust: unknown module `{}`", cfg.kind);
            None
        }
    }
}

/// Spawn every entry in a slot list, dropping unknown / unfinished types.
pub fn spawn_all(mods: &[ModuleCfg], cx: &mut Context<AppBar>) -> Vec<SlotItem> {
    mods.iter().filter_map(|m| spawn_module(m, cx)).collect()
}
