//! TOML config for crust: theme, bar chrome, slot classes, and module lists.
//!
//! # File layout
//!
//! ```toml
//! theme = "mocha"
//!
//! [colors]
//! accent = "#f38ba8"
//!
//! [bar]
//! position = "top"
//! height = 36
//! class = "w-full h-full items-center bg-surface/90 text-text"
//!
//! [bar.start]
//! class = "flex items-center gap-2"
//! container_class = "justify-start px-2"
//!
//! [[start]]
//! type = "workspaces"
//! class = "flex gap-1"
//!
//! [[center]]
//! type = "clock"
//! class = "px-3 text-sm"
//! format = "%H:%M"
//!
//! [[end]]
//! type = "volume"
//! class = "px-2"
//! ```
//!
//! `[bar.center]` is slot chrome. `[[center]]` is the module list. They are
//! not the same key.
//!
//! Missing file, empty file, parse error, or a file with no modules uses
//! [`BASE_THEME`]. Only the empty/missing case is written to disk.
//!
//! [`Config::load_or_default`] calls [`Config::with_theme_colors`], so
//! `colors` always contains `surface`, `text`, `accent`, and `muted`.
//! A `[colors]` entry in the file wins over the palette.

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

/// Where the layer-shell surface is anchored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BarPosition {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
}

/// Resolved palette. Hex digits, no leading `#`.
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub surface: String,
    pub text: String,
    pub accent: String,
    pub muted: String,
}

/// Built-in bar used when the user has no usable config.
pub const BASE_THEME: &str = r#"
theme = "mocha"

[bar]
position = "top"
height = 36
class = "w-full h-full items-center bg-surface/90 text-text px-1"

[bar.start]
class = "flex items-center gap-2"
container_class = "justify-start px-2"

[bar.center]
class = "flex items-center gap-2"
container_class = "justify-center"

[bar.end]
class = "flex items-center gap-2"
container_class = "justify-end px-2"

[[start]]
type = "workspaces"
class = "flex gap-1 text-text"
[start.parts]
item = "px-2 rounded text-muted"
active = "px-2 rounded bg-accent text-surface"

[[start]]
type = "window"
class = "px-2 text-sm text-text"
max_length = 40

[[center]]
type = "clock"
class = "px-3 text-sm text-text"
format = "%H:%M"

[[end]]
type = "cpu"
class = "px-2 text-sm text-text"

[[end]]
type = "memory"
class = "px-2 text-sm text-text"

[[end]]
type = "volume"
class = "px-2 text-sm text-text"

[[end]]
type = "wifi"
class = "px-2 text-sm text-text"

[[end]]
type = "battery"
class = "px-2 text-sm text-text"

[[end]]
type = "notifications"
class = "px-2 text-sm text-text"
"#;

/// Root document.
///
/// Missing slot lists become empty vecs. Missing `[bar]` uses [`BarCfg`] defaults.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// `mocha` (default), `latte` / `light`, `zinc` / `dark`. Unknown names use mocha.
    #[serde(default = "default_theme_name")]
    pub theme: String,
    #[serde(default)]
    pub bar: BarCfg,
    #[serde(default)]
    pub start: Vec<ModuleCfg>,
    #[serde(default)]
    pub center: Vec<ModuleCfg>,
    #[serde(default)]
    pub end: Vec<ModuleCfg>,
    /// Token overrides, e.g. `accent = "#f38ba8"`. Wins over [`Self::palette`].
    #[serde(default)]
    pub colors: HashMap<String, String>,
}

fn default_theme_name() -> String {
    "mocha".into()
}

impl Config {
    /// Parse a TOML file. Does not substitute the built-in theme or fill theme colors.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let raw = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::Io(path.display().to_string(), e))?;
        toml::from_str(&raw).map_err(|e| ConfigError::Parse(path.display().to_string(), e))
    }

    /// Load `path`, or the built-in theme if it is missing, empty, invalid, or has no modules.
    ///
    /// Writes [`BASE_THEME`] only when the file is missing or empty.
    /// The returned config has theme tokens filled in via [`Self::with_theme_colors`].
    pub fn load_or_default(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        if !path.exists() {
            eprintln!("crust: no {}; writing built-in theme", path.display());
            return write_default(path);
        }
        let raw = std::fs::read_to_string(path).unwrap_or_default();
        if raw.trim().is_empty() {
            eprintln!("crust: {} is empty; writing built-in theme", path.display());
            return write_default(path);
        }
        match toml::from_str::<Self>(&raw) {
            Ok(cfg) if cfg.is_usable() => cfg.with_theme_colors(),
            Ok(_) => {
                eprintln!(
                    "crust: {} has no modules; using built-in theme",
                    path.display()
                );
                builtin_theme().with_theme_colors()
            }
            Err(e) => {
                eprintln!("crust: parse {}: {e}; using built-in theme", path.display());
                builtin_theme().with_theme_colors()
            }
        }
    }

    /// False when every slot list is empty. That bar would paint nothing.
    pub fn is_usable(&self) -> bool {
        !self.start.is_empty() || !self.center.is_empty() || !self.end.is_empty()
    }

    /// Palette for `theme`, before `[colors]` overrides.
    pub fn palette(&self) -> Theme {
        palette(&self.theme)
    }

    /// Hex without `#` for `surface`, `text`, `accent`, or `muted`.
    pub fn color(&self, token: &str) -> String {
        if let Some(hex) = self.colors.get(token) {
            return hex.trim_start_matches('#').to_string();
        }
        let p = self.palette();
        match token {
            "surface" => p.surface,
            "text" => p.text,
            "accent" => p.accent,
            "muted" => p.muted,
            _ => p.text,
        }
    }

    /// Insert palette hexes for any token the user did not set in `[colors]`.
    ///
    /// After this, `colors["surface"]` is safe to pass to `apply_class_with`.
    /// A `[colors]` entry already in the file is left alone.
    pub fn with_theme_colors(mut self) -> Self {
        let p = self.palette();
        self.colors.entry("surface".into()).or_insert(p.surface);
        self.colors.entry("text".into()).or_insert(p.text);
        self.colors.entry("accent".into()).or_insert(p.accent);
        self.colors.entry("muted".into()).or_insert(p.muted);
        self
    }
}

fn palette(name: &str) -> Theme {
    match name {
        "latte" | "light" => Theme {
            name: "latte".into(),
            surface: "eff1f5".into(),
            text: "4c4f69".into(),
            accent: "1e66f5".into(),
            muted: "8c8fa1".into(),
        },
        "zinc" | "dark" => Theme {
            name: "zinc".into(),
            surface: "18181b".into(),
            text: "fafafa".into(),
            accent: "3b82f6".into(),
            muted: "a1a1aa".into(),
        },
        _ => Theme {
            name: "mocha".into(),
            surface: "1e1e2e".into(),
            text: "cdd6f4".into(),
            accent: "89b4fa".into(),
            muted: "6c7086".into(),
        },
    }
}

fn builtin_theme() -> Config {
    toml::from_str(BASE_THEME).expect("builtin theme")
}

fn write_default(path: &Path) -> Config {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Err(e) = std::fs::write(path, BASE_THEME) {
        eprintln!("crust: could not write {}: {e}", path.display());
    }
    builtin_theme().with_theme_colors()
}

/// Failed to read or parse a config file.
#[derive(Debug)]
pub enum ConfigError {
    Io(String, std::io::Error),
    Parse(String, toml::de::Error),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(path, e) => write!(f, "read {path}: {e}"),
            Self::Parse(path, e) => write!(f, "parse {path}: {e}"),
        }
    }
}

impl std::error::Error for ConfigError {}

/// Layer-shell geometry and classes for the bar and three columns.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct BarCfg {
    #[serde(default)]
    pub position: BarPosition,
    /// Surface height in logical px. Must match the exclusive zone.
    #[serde(default = "default_height")]
    pub height: f32,
    #[serde(default)]
    pub class: String,
    #[serde(default)]
    pub start: SlotCfg,
    #[serde(default)]
    pub center: SlotCfg,
    #[serde(default)]
    pub end: SlotCfg,
}

fn default_height() -> f32 {
    36.0
}

/// Classes for one column (`[bar.start]`, `[bar.center]`, `[bar.end]`).
#[derive(Debug, Clone, Deserialize, Default)]
pub struct SlotCfg {
    /// Pack: `#start` / `#center` / `#end`.
    #[serde(default)]
    pub class: String,
    /// Column: `#start-container` and friends. crust-tw, on top of structural `flex-1`.
    #[serde(default)]
    pub container_class: String,
}

/// One `[[start]]` / `[[center]]` / `[[end]]` entry.
#[derive(Debug, Clone, Deserialize)]
pub struct ModuleCfg {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub class: String,
    #[serde(default)]
    pub parts: HashMap<String, String>,
    #[serde(flatten)]
    pub extra: HashMap<String, toml::Value>,
}

impl ModuleCfg {
    /// String value of a flattened extra key, if present and a string.
    pub fn extra_str(&self, key: &str) -> Option<&str> {
        self.extra.get(key).and_then(|v| v.as_str())
    }
}

/// Which column a module was declared in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    Start,
    Center,
    End,
}
