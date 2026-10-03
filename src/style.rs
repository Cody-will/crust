//! crust-tw: Tailwind-like class strings → GPUI [`Styled`] methods.
//!
//! Widgets and config pass a class string. This module turns
//! `"px-3 rounded-md bg-zinc-900 text-sm"` into builder calls on a `div()`.
//!
//! # Supported tokens
//!
//! ## Layout
//! `flex`, `flex-row`, `flex-col`, `flex-1`, `flex-none`,
//! `items-start`, `items-center`, `items-end`,
//! `justify-start`, `justify-center`, `justify-between`,
//! `justify-end`, `justify-around`,
//! `overflow-hidden`, `overflow-x-hidden`, `overflow-y-hidden`,
//! `w-full`, `h-full`, `size-full`, `w-auto`, `h-auto`,
//! `cursor-pointer`, `cursor-default`
//!
//! ## Type
//! `text-xs` … `text-3xl`, `text-center`, `text-left`, `text-right`,
//! `truncate`,
//! `font-thin`, `font-light`, `font-normal`, `font-medium`, `font-semibold`,
//! `font-bold`
//!
//! ## Radius / border
//! `rounded-none`, `rounded-sm`, `rounded`, `rounded-md`, `rounded-lg`,
//! `rounded-xl`, `rounded-full`,
//! `border`, `border-0`, `border-2`
//!
//! ## Spacing (n × 4px, or `p-[10px]`)
//! `p-*` `px-*` `py-*` `pt-*` `pr-*` `pb-*` `pl-*`
//! `m-*` `mx-*` `my-*` `mt-*` `mr-*` `mb-*` `ml-*`
//! `gap-*` `gap-x-*` `gap-y-*`
//!
//! ## Sizing scale
//! `w-*` `h-*` `min-w-*` `min-h-*` `max-w-*` `max-h-*` (same n × 4px scale)
//!
//! ## Color
//! Arbitrary: `bg-[#11111b]`, `text-[#cdd6f4]`, `border-[#89b4fa]`
//! (3 / 6 / 8 hex digits)
//!
//! Named (Tailwind-ish): `bg-black`, `text-white`, `bg-zinc-900`,
//! `text-red-400`, … families `slate` `gray` `zinc` `neutral` `stone`
//! `red` `orange` `amber` `yellow` `lime` `green` `emerald` `teal` `cyan`
//! `sky` `blue` `indigo` `violet` `purple` `fuchsia` `pink` `rose`
//! shades `50` `100` `200` `300` `400` `500` `600` `700` `800` `900` `950`
//!
//! Alpha on that color only: `bg-zinc-800/50`, `text-white/80`,
//! `bg-[#11111b]/70`. `/50` means 50%. This does not fade children.
//!
//! Theme tokens, after [`set_theme`]: `bg-surface`, `text-text`, `bg-accent`,
//! `text-muted`, `border-accent`, with optional `/50`. [`apply_class`] and
//! [`apply_class_with`] both see them. `border-2` is a width, not a color.
//!
//! Also `bg-transparent`, `text-transparent`.
//!
//! ## Element opacity
//! `opacity-0`, `opacity-25`, `opacity-50`, `opacity-75`, `opacity-100`
//! fade the whole element, including text and children.
//!
//! ## Not implemented
//! `hover:`, `dark:`, `focus:`, selectors, `@apply`. Unknown tokens log and skip.
//!
//! # Examples
//!
//! ```ignore
//! set_theme(config.colors.clone());
//! box_with("px-2 rounded bg-accent text-surface", ["1"]);
//! apply_class_with(div(), "bg-surface/90 text-text", &config.colors);
//! ```

use std::collections::HashMap;
use std::sync::Mutex;

use gpui_kit::{base::StyledExt, *};

static THEME: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);

/// Install theme tokens for every later [`apply_class`] call.
///
/// Call from [`AppBar::new`](crate::app_bar::AppBar::new) and reload, after
/// [`Config::with_theme_colors`](crate::config::Config::with_theme_colors).
pub fn set_theme(colors: HashMap<String, String>) {
    if let Ok(mut slot) = THEME.lock() {
        *slot = Some(colors);
    }
}

fn theme_hex(name: &str) -> Option<u32> {
    let slot = THEME.lock().ok()?;
    let colors = slot.as_ref()?;
    let hex = colors.get(name)?;
    u32::from_str_radix(hex.trim_start_matches('#'), 16).ok()
}

/// Apply a whitespace-separated class string to any [`Styled`] element.
///
/// Left to right. Later tokens win when they set the same property.
/// Theme names resolve after [`set_theme`].
pub fn apply_class<E: Styled>(el: E, class: &str) -> E {
    class.split_ascii_whitespace().fold(el, apply_token)
}

/// [`apply_class`] plus an explicit color map. The map is checked before [`set_theme`].
pub fn apply_class_with<E: Styled>(
    el: E,
    class: &str,
    colors: &HashMap<String, String>,
) -> E {
    class
        .split_ascii_whitespace()
        .fold(el, |el, token| apply_token_with(el, token, colors))
}

/// `div()` + [`apply_class`] + children. Usual widget root.
pub fn box_with(
    class: &str,
    children: impl IntoIterator<Item = impl IntoElement>,
) -> Div {
    apply_class(div(), class).children(children)
}

/// [`box_with`] that also resolves tokens from `colors`.
pub fn box_with_colors(
    class: &str,
    colors: &HashMap<String, String>,
    children: impl IntoIterator<Item = impl IntoElement>,
) -> Div {
    apply_class_with(div(), class, colors).children(children)
}

fn apply_token<E: Styled>(el: E, token: &str) -> E {
    match token {
        "flex" | "flex-row" => el.flex(),
        "flex-col" => el.flex_col(),
        "flex-1" => el.flex_1(),
        "flex-none" => el.flex_none(),
        "items-start" => el.items_start(),
        "items-center" => el.items_center(),
        "items-end" => el.items_end(),
        "justify-start" => el.justify_start(),
        "justify-center" => el.justify_center(),
        "justify-between" => el.justify_between(),
        "justify-end" => el.justify_end(),
        "justify-around" => el.justify_around(),
        "overflow-hidden" => el.overflow_hidden(),
        "overflow-x-hidden" => el.overflow_x_hidden(),
        "overflow-y-hidden" => el.overflow_y_hidden(),
        "w-full" => el.w_full(),
        "h-full" => el.h_full(),
        "size-full" => el.size_full(),
        "w-auto" => el.w_auto(),
        "h-auto" => el.h_auto(),
        "cursor-pointer" => el.cursor_pointer(),
        "cursor-default" => el.cursor_default(),
        "rounded-none" => el.rounded_none(),
        "rounded-sm" => el.rounded_sm(),
        "rounded" | "rounded-md" => el.rounded_md(),
        "rounded-lg" => el.rounded_lg(),
        "rounded-xl" => el.rounded_xl(),
        "rounded-full" => el.rounded_full(),
        "text-xs" => el.text_xs(),
        "text-sm" => el.text_sm(),
        "text-base" => el.text_base(),
        "text-lg" => el.text_lg(),
        "text-xl" => el.text_xl(),
        "text-2xl" => el.text_2xl(),
        "text-3xl" => el.text_3xl(),
        "text-left" => el.text_left(),
        "text-center" => el.text_center(),
        "text-right" => el.text_right(),
        "truncate" => el.truncate(),
        "font-thin" => el.font_thin(),
        "font-light" => el.font_light(),
        "font-normal" => el.font_normal(),
        "font-medium" => el.font_medium(),
        "font-semibold" => el.font_semibold(),
        "font-bold" => el.font_bold(),
        "border" => el.border_1(),
        "border-0" => el.border_0(),
        "border-2" => el.border_2(),
        "opacity-0" => el.opacity(0.0),
        "opacity-25" => el.opacity(0.25),
        "opacity-50" => el.opacity(0.5),
        "opacity-75" => el.opacity(0.75),
        "opacity-100" => el.opacity(1.0),
        other => apply_dynamic(el, other),
    }
}

fn apply_token_with<E: Styled>(el: E, token: &str, colors: &HashMap<String, String>) -> E {
    if is_color_token(token) {
        apply_dynamic_with(el, token, colors)
    } else {
        apply_token(el, token)
    }
}

fn is_color_token(token: &str) -> bool {
    match token {
        "border" | "border-0" | "border-2" | "text-xs" | "text-sm" | "text-base" | "text-lg"
        | "text-xl" | "text-2xl" | "text-3xl" | "text-left" | "text-center" | "text-right" => false,
        _ => token.starts_with("bg-") || token.starts_with("text-") || token.starts_with("border-"),
    }
}

fn apply_dynamic<E: Styled>(el: E, token: &str) -> E {
    if let Some(n) = spacing("px-", token) {
        return el.px(n);
    }
    if let Some(n) = spacing("py-", token) {
        return el.py(n);
    }
    if let Some(n) = spacing("pt-", token) {
        return el.pt(n);
    }
    if let Some(n) = spacing("pr-", token) {
        return el.pr(n);
    }
    if let Some(n) = spacing("pb-", token) {
        return el.pb(n);
    }
    if let Some(n) = spacing("pl-", token) {
        return el.pl(n);
    }
    if let Some(n) = spacing("p-", token) {
        return el.p(n);
    }
    if let Some(n) = spacing("mx-", token) {
        return el.mx(n);
    }
    if let Some(n) = spacing("my-", token) {
        return el.my(n);
    }
    if let Some(n) = spacing("mt-", token) {
        return el.mt(n);
    }
    if let Some(n) = spacing("mr-", token) {
        return el.mr(n);
    }
    if let Some(n) = spacing("mb-", token) {
        return el.mb(n);
    }
    if let Some(n) = spacing("ml-", token) {
        return el.ml(n);
    }
    if let Some(n) = spacing("m-", token) {
        return el.m(n);
    }
    if let Some(n) = spacing("gap-x-", token) {
        return el.gap_x(n);
    }
    if let Some(n) = spacing("gap-y-", token) {
        return el.gap_y(n);
    }
    if let Some(n) = spacing("gap-", token) {
        return el.gap(n);
    }
    if let Some(n) = spacing("min-w-", token) {
        return el.min_w(n);
    }
    if let Some(n) = spacing("min-h-", token) {
        return el.min_h(n);
    }
    if let Some(n) = spacing("max-w-", token) {
        return el.max_w(n);
    }
    if let Some(n) = spacing("max-h-", token) {
        return el.max_h(n);
    }
    if let Some(n) = spacing("w-", token) {
        return el.w(n);
    }
    if let Some(n) = spacing("h-", token) {
        return el.h(n);
    }
    if let Some(c) = color_token("bg-", token) {
        return el.bg(c);
    }
    if let Some(c) = color_token("text-", token) {
        return el.text_color(c);
    }
    if let Some(c) = color_token("border-", token) {
        return el.border_color(c);
    }
    eprintln!("crust: unknown class `{token}`");
    el
}

fn apply_dynamic_with<E: Styled>(el: E, token: &str, colors: &HashMap<String, String>) -> E {
    if let Some(c) = color_token_with("bg-", token, colors) {
        return el.bg(c);
    }
    if let Some(c) = color_token_with("text-", token, colors) {
        return el.text_color(c);
    }
    if let Some(c) = color_token_with("border-", token, colors) {
        return el.border_color(c);
    }
    apply_dynamic(el, token)
}

/// `px-3` → 12px, `px-[10px]` → 10px.
fn spacing(prefix: &str, token: &str) -> Option<Pixels> {
    let rest = token.strip_prefix(prefix)?;
    if rest == "full" || rest == "auto" || rest == "px" {
        return None;
    }
    if let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
        let n = inner.trim_end_matches("px").parse::<f32>().ok()?;
        return Some(px(n));
    }
    let n: f32 = rest.parse().ok()?;
    Some(px(n * 4.0))
}

fn color_token(prefix: &str, token: &str) -> Option<Rgba> {
    let rest = token.strip_prefix(prefix)?;
    let (color_part, alpha) = split_alpha(rest);
    let rgb_hex = color_rgb(color_part)?;
    Some(with_alpha(rgb_hex, alpha))
}

fn color_token_with(prefix: &str, token: &str, colors: &HashMap<String, String>) -> Option<Rgba> {
    let rest = token.strip_prefix(prefix)?;
    let (color_part, alpha) = split_alpha(rest);
    let rgb_hex = color_rgb_with(color_part, colors)?;
    Some(with_alpha(rgb_hex, alpha))
}

/// `"zinc-800/50"` → `("zinc-800", 0.5)`, `"zinc-800"` → `(…, 1.0)`.
fn split_alpha(rest: &str) -> (&str, f32) {
    match rest.rsplit_once('/') {
        Some((color, spec)) => {
            let a = spec.parse::<f32>().ok().map(|n| {
                if n > 1.0 {
                    n / 100.0
                } else {
                    n
                }
            });
            (color, a.unwrap_or(1.0))
        }
        None => (rest, 1.0),
    }
}

fn color_rgb(name: &str) -> Option<u32> {
    if let Some(hex) = name.strip_prefix("[#").and_then(|s| s.strip_suffix(']')) {
        return match hex.len() {
            3 => {
                let n = u32::from_str_radix(hex, 16).ok()?;
                let r = (n >> 8) & 0xF;
                let g = (n >> 4) & 0xF;
                let b = n & 0xF;
                Some((r << 20) | (r << 16) | (g << 12) | (g << 8) | (b << 4) | b)
            }
            6 | 8 => u32::from_str_radix(&hex[..6], 16).ok(),
            _ => None,
        };
    }
    if let Some(hex) = theme_hex(name) {
        return Some(hex);
    }
    match name {
        "transparent" => Some(0x000000),
        "black" => Some(0x000000),
        "white" => Some(0xffffff),
        other => palette_hex(other),
    }
}

fn color_rgb_with(name: &str, colors: &HashMap<String, String>) -> Option<u32> {
    if let Some(hex) = colors.get(name) {
        return u32::from_str_radix(hex.trim_start_matches('#'), 16).ok();
    }
    color_rgb(name)
}

fn with_alpha(rgb_hex: u32, frac: f32) -> Rgba {
    let a = (frac.clamp(0.0, 1.0) * 255.0).round() as u32;
    rgba((rgb_hex << 8) | a)
}

fn palette_hex(name: &str) -> Option<u32> {
    let (fam, shade) = name.rsplit_once('-')?;
    let i = match shade {
        "50" => 0,
        "100" => 1,
        "200" => 2,
        "300" => 3,
        "400" => 4,
        "500" => 5,
        "600" => 6,
        "700" => 7,
        "800" => 8,
        "900" => 9,
        "950" => 10,
        _ => return None,
    };
    let row: &[u32; 11] = match fam {
        "slate" => &[
            0xf8fafc, 0xf1f5f9, 0xe2e8f0, 0xcbd5e1, 0x94a3b8, 0x64748b, 0x475569, 0x334155,
            0x1e293b, 0x0f172a, 0x020617,
        ],
        "gray" => &[
            0xf9fafb, 0xf3f4f6, 0xe5e7eb, 0xd1d5db, 0x9ca3af, 0x6b7280, 0x4b5563, 0x374151,
            0x1f2937, 0x111827, 0x030712,
        ],
        "zinc" => &[
            0xfafafa, 0xf4f4f5, 0xe4e4e7, 0xd4d4d8, 0xa1a1aa, 0x71717a, 0x52525b, 0x3f3f46,
            0x27272a, 0x18181b, 0x09090b,
        ],
        "neutral" => &[
            0xfafafa, 0xf5f5f5, 0xe5e5e5, 0xd4d4d4, 0xa3a3a3, 0x737373, 0x525252, 0x404040,
            0x262626, 0x171717, 0x0a0a0a,
        ],
        "stone" => &[
            0xfafaf9, 0xf5f5f4, 0xe7e5e4, 0xd6d3d1, 0xa8a29e, 0x78716c, 0x57534e, 0x44403c,
            0x292524, 0x1c1917, 0x0c0a09,
        ],
        "red" => &[
            0xfef2f2, 0xfee2e2, 0xfecaca, 0xfca5a5, 0xf87171, 0xef4444, 0xdc2626, 0xb91c1c,
            0x991b1b, 0x7f1d1d, 0x450a0a,
        ],
        "orange" => &[
            0xfff7ed, 0xffedd5, 0xfed7aa, 0xfdba74, 0xfb923c, 0xf97316, 0xea580c, 0xc2410c,
            0x9a3412, 0x7c2d12, 0x431407,
        ],
        "amber" => &[
            0xfffbeb, 0xfef3c7, 0xfde68a, 0xfcd34d, 0xfbbf24, 0xf59e0b, 0xd97706, 0xb45309,
            0x92400e, 0x78350f, 0x451a03,
        ],
        "yellow" => &[
            0xfefce8, 0xfef9c3, 0xfef08a, 0xfde047, 0xfacc15, 0xeab308, 0xca8a04, 0xa16207,
            0x854d0e, 0x713f12, 0x422006,
        ],
        "lime" => &[
            0xf7fee7, 0xecfccb, 0xd9f99d, 0xbef264, 0xa3e635, 0x84cc16, 0x65a30d, 0x4d7c0f,
            0x3f6212, 0x365314, 0x1a2e05,
        ],
        "green" => &[
            0xf0fdf4, 0xdcfce7, 0xbbf7d0, 0x86efac, 0x4ade80, 0x22c55e, 0x16a34a, 0x15803d,
            0x166534, 0x14532d, 0x052e16,
        ],
        "emerald" => &[
            0xecfdf5, 0xd1fae5, 0xa7f3d0, 0x6ee7b7, 0x34d399, 0x10b981, 0x059669, 0x047857,
            0x065f46, 0x064e3b, 0x022c22,
        ],
        "teal" => &[
            0xf0fdfa, 0xccfbf1, 0x99f6e4, 0x5eead4, 0x2dd4bf, 0x14b8a6, 0x0d9488, 0x0f766e,
            0x115e59, 0x134e4a, 0x042f2e,
        ],
        "cyan" => &[
            0xecfeff, 0xcffafe, 0xa5f3fc, 0x67e8f9, 0x22d3ee, 0x06b6d4, 0x0891b2, 0x0e7490,
            0x155e75, 0x164e63, 0x083344,
        ],
        "sky" => &[
            0xf0f9ff, 0xe0f2fe, 0xbae6fd, 0x7dd3fc, 0x38bdf8, 0x0ea5e9, 0x0284c7, 0x0369a1,
            0x075985, 0x0c4a6e, 0x082f49,
        ],
        "blue" => &[
            0xeff6ff, 0xdbeafe, 0xbfdbfe, 0x93c5fd, 0x60a5fa, 0x3b82f6, 0x2563eb, 0x1d4ed8,
            0x1e40af, 0x1e3a8a, 0x172554,
        ],
        "indigo" => &[
            0xeef2ff, 0xe0e7ff, 0xc7d2fe, 0xa5b4fc, 0x818cf8, 0x6366f1, 0x4f46e5, 0x4338ca,
            0x3730a3, 0x312e81, 0x1e1b4b,
        ],
        "violet" => &[
            0xf5f3ff, 0xede9fe, 0xddd6fe, 0xc4b5fd, 0xa78bfa, 0x8b5cf6, 0x7c3aed, 0x6d28d9,
            0x5b21b6, 0x4c1d95, 0x2e1065,
        ],
        "purple" => &[
            0xfaf5ff, 0xf3e8ff, 0xe9d5ff, 0xd8b4fe, 0xc084fc, 0xa855f7, 0x9333ea, 0x7e22ce,
            0x6b21a8, 0x581c87, 0x3b0764,
        ],
        "fuchsia" => &[
            0xfdf4ff, 0xfae8ff, 0xf5d0fe, 0xf0abfc, 0xe879f9, 0xd946ef, 0xc026d3, 0xa21caf,
            0x86198f, 0x701a75, 0x4a044e,
        ],
        "pink" => &[
            0xfdf2f8, 0xfce7f3, 0xfbcfe8, 0xf9a8d4, 0xf472b6, 0xec4899, 0xdb2777, 0xbe185d,
            0x9d174d, 0x831843, 0x500724,
        ],
        "rose" => &[
            0xfff1f2, 0xffe4e6, 0xfecdd3, 0xfda4af, 0xfb7185, 0xf43f5e, 0xe11d48, 0xbe123c,
            0x9f1239, 0x881337, 0x4c0519,
        ],
        _ => return None,
    };
    Some(row[i])
}
