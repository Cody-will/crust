//! Layout shell: three columns, each with a container and a pack.
//!
//! ```text
//! #bar
//!   #start-container
//!     #start
//!   #center-container
//!     #center
//!   #end-container
//!     #end
//! ```
//!
//! Class strings are crust-tw. Theme tokens resolve through [`apply_class_with`]
//! and the map passed from [`crate::app_bar::AppBar`].
//!
//! Structural defaults (always applied, then config on top):
//! - `#bar`: `flex items-center size-full`
//! - each `*-container`: `flex flex-1 w-full min-w-0 items-center`
//! - each pack: `flex items-center` when `[bar.*].class` is empty

use std::collections::HashMap;

use gpui_kit::*;

use crate::style::apply_class_with;

const PACK: &str = "flex items-center";

/// Start / center / end row. Build in `render`, do not store across frames.
pub struct Bar {
    class: String,
    start_class: String,
    center_class: String,
    end_class: String,
    start_container: String,
    center_container: String,
    end_container: String,
    start: Vec<AnyElement>,
    center: Vec<AnyElement>,
    end: Vec<AnyElement>,
    colors: HashMap<String, String>,
}

impl Bar {
    /// Empty bar with the given outer crust-tw class (`[bar].class`).
    pub fn new(class: impl Into<String>) -> Self {
        Self {
            class: class.into(),
            start_class: String::new(),
            center_class: String::new(),
            end_class: String::new(),
            start_container: String::new(),
            center_container: String::new(),
            end_container: String::new(),
            start: Vec::new(),
            center: Vec::new(),
            end: Vec::new(),
            colors: HashMap::new(),
        }
    }

    /// Theme map from [`crate::config::Config::with_theme_colors`].
    pub fn colors(mut self, colors: HashMap<String, String>) -> Self {
        self.colors = colors;
        self
    }

    /// Pack classes for the left modules (`[bar.start].class` → `#start`).
    pub fn start_class(mut self, c: impl Into<String>) -> Self {
        self.start_class = c.into();
        self
    }

    /// Pack classes for the middle modules (`[bar.center].class` → `#center`).
    pub fn center_class(mut self, c: impl Into<String>) -> Self {
        self.center_class = c.into();
        self
    }

    /// Pack classes for the right modules (`[bar.end].class` → `#end`).
    pub fn end_class(mut self, c: impl Into<String>) -> Self {
        self.end_class = c.into();
        self
    }

    /// Column classes (`[bar.start].container_class` → `#start-container`).
    pub fn start_container_class(mut self, c: impl Into<String>) -> Self {
        self.start_container = c.into();
        self
    }

    /// Column classes (`[bar.center].container_class` → `#center-container`).
    pub fn center_container_class(mut self, c: impl Into<String>) -> Self {
        self.center_container = c.into();
        self
    }

    /// Column classes (`[bar.end].container_class` → `#end-container`).
    pub fn end_container_class(mut self, c: impl Into<String>) -> Self {
        self.end_container = c.into();
        self
    }

    /// Append a child to the left pack. Call once per module.
    pub fn start(mut self, child: impl IntoElement) -> Self {
        self.start.push(child.into_any_element());
        self
    }

    /// Append a child to the middle pack.
    pub fn center(mut self, child: impl IntoElement) -> Self {
        self.center.push(child.into_any_element());
        self
    }

    /// Append a child to the right pack.
    pub fn end(mut self, child: impl IntoElement) -> Self {
        self.end.push(child.into_any_element());
        self
    }
}

fn container_tokens(extra: &str) -> String {
    format!("flex flex-1 w-full min-w-0 items-center {extra}")
}

fn pack_tokens(configured: &str) -> &str {
    if configured.is_empty() {
        PACK
    } else {
        configured
    }
}

fn slot(
    name: &'static str,
    pack: &str,
    container: &str,
    colors: &HashMap<String, String>,
    children: Vec<AnyElement>,
) -> Stateful<Div> {
    apply_class_with(
        div().id(SharedString::from(format!("{name}-container"))),
        &container_tokens(container),
        colors,
    )
    .child(
        apply_class_with(
            div().id(SharedString::from(name.to_string())),
            pack_tokens(pack),
            colors,
        )
        .children(children),
    )
}

impl IntoElement for Bar {
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        apply_class_with(
            div().id("bar").flex().items_center().size_full(),
            &self.class,
            &self.colors,
        )
        .child(slot(
            "start",
            &self.start_class,
            &self.start_container,
            &self.colors,
            self.start,
        ))
        .child(slot(
            "center",
            &self.center_class,
            &self.center_container,
            &self.colors,
            self.center,
        ))
        .child(slot(
            "end",
            &self.end_class,
            &self.end_container,
            &self.colors,
            self.end,
        ))
    }
}
