//! Network overlay. Backdrop click and the close button dismiss it.
//!
//! nmcli only. A saved profile connects with `connection up` and does not
//! ask for a password. A new secured SSID uses the field. Forget deletes
//! the profile.
//!
//! Click handlers are `Fn`, so every captured `String` is cloned inside
//! the body. A move out of the closure fails to compile.

use std::process::Command;

use gpui_kit::*;

use crate::config::ModuleCfg;
use crate::style::apply_class;

#[derive(Clone, Debug)]
struct Net {
    ssid: String,
    signal: u8,
    security: String,
    active: bool,
    saved: bool,
}

pub struct WifiDrawer {
    cfg: ModuleCfg,
    nets: Vec<Net>,
    password: String,
    selected: Option<String>,
    status: SharedString,
    device: String,
}

impl WifiDrawer {
    pub fn open(cfg: ModuleCfg, cx: &mut Context<Self>) -> Self {
        let device = wifi_device();
        let nets = scan(&device);
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_secs(8))
                    .await;
                let _ = this.update(cx, |this, cx| {
                    this.nets = scan(&this.device);
                    cx.notify();
                });
            }
        })
        .detach();
        Self {
            cfg,
            nets,
            password: String::new(),
            selected: None,
            status: "".into(),
            device,
        }
    }

    fn part<'a>(&'a self, key: &str, fallback: &'a str) -> &'a str {
        self.cfg.parts.get(key).map(String::as_str).unwrap_or(fallback)
    }
}

impl Render for WifiDrawer {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let drawer_class = self
            .part("drawer", "w-[300px] p-2 gap-1 rounded-md bg-surface text-text")
            .to_string();
        let row_class = self.part("row", "px-2 py-1 rounded text-text").to_string();
        let active_class = self
            .part("active", "px-2 py-1 rounded bg-accent/30 text-text")
            .to_string();
        let connect_class = self
            .part("connect", "mt-1 px-2 py-1 rounded-md bg-accent text-surface")
            .to_string();

        let mut panel = apply_class(div().id("wifi-panel").flex().flex_col(), &drawer_class)
            .on_click(|_, _, cx| cx.stop_propagation());

        panel = panel.child(
            div()
                .flex()
                .justify_between()
                .items_center()
                .child("Wi-Fi")
                .child(
                    div()
                        .id("wifi-close")
                        .cursor_pointer()
                        .px_2()
                        .on_click(cx.listener(|_, _, window, _| window.remove_window()))
                        .child("close"),
                ),
        );

        for net in self.nets.clone() {
            let ssid = net.ssid.clone();
            let ssid_click = ssid.clone();
            let selected = self.selected.as_deref() == Some(ssid.as_str());
            let class = if net.active || selected {
                active_class.as_str()
            } else {
                row_class.as_str()
            };
            let mark = if net.active {
                "connected"
            } else if net.security.is_empty() || net.security == "--" {
                "open"
            } else if net.saved {
                "saved"
            } else {
                "locked"
            };
            panel = panel.child(
                apply_class(div().id(SharedString::from(format!("net-{ssid}"))), class)
                    .flex()
                    .justify_between()
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.selected = Some(ssid_click.clone());
                        this.password.clear();
                        this.status = "".into();
                        cx.notify();
                    }))
                    .child(div().flex_1().min_w_0().overflow_hidden().child(div().truncate().child(ssid)), )
                    .child(format!("{}%  {mark}", net.signal)),
            );
        }

        if let Some(ssid) = self.selected.clone() {
            let net = self.nets.iter().find(|n| n.ssid == ssid).cloned();
            let locked = net
                .as_ref()
                .is_some_and(|n| !n.security.is_empty() && n.security != "--");
            let saved = net.as_ref().is_some_and(|n| n.saved);
            let active = net.as_ref().is_some_and(|n| n.active);

            if locked && !saved && !active {
                let masked = "*".repeat(self.password.len());
                panel = panel.child(
                    div()
                        .id("wifi-password")
                        .px_2()
                        .py_1()
                        .border_1()
                        .rounded_md()
                        .child(if masked.is_empty() {
                            "password".to_string()
                        } else {
                            masked
                        })
                        .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                            match event.keystroke.key.as_str() {
                                "backspace" => {
                                    this.password.pop();
                                }
                                "enter" => {}
                                key if key.chars().count() == 1 && !event.keystroke.modifiers.control => {
                                    this.password.push_str(key);
                                }
                                _ => {}
                            }
                            cx.notify();
                        })),
                );
            }

            let ssid_action = ssid.clone();
            let password = self.password.clone();
            let device = self.device.clone();
            let label = if active {
                "disconnect"
            } else if saved || !locked {
                "connect"
            } else {
                "connect with password"
            };
            panel = panel.child(
                apply_class(div().id("wifi-action"), &connect_class)
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.status = if active {
                            disconnect(&device)
                        } else {
                            connect(&ssid_action, &password, saved || !locked)
                        }
                        .into();
                        this.nets = scan(&this.device);
                        cx.notify();
                    }))
                    .child(label),
            );

            if saved {
                let ssid_forget = ssid.clone();
                let device = self.device.clone();
                panel = panel.child(
                    div()
                        .id("wifi-forget")
                        .px_2()
                        .cursor_pointer()
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.status = forget(&ssid_forget).into();
                            this.nets = scan(&device);
                            cx.notify();
                        }))
                        .child("forget"),
                );
            }
        }

        if !self.status.is_empty() {
            panel = panel.child(div().text_xs().child(self.status.to_string()));
        }

        div()
            .id("wifi-backdrop")
            .size_full()
            .flex()
            .justify_end()
            .pt(px(40.))
            .pr(px(8.))
            .on_click(cx.listener(|_, _, window, _| window.remove_window()))
            .child(panel)
            .on_key_down(cx.listener(|_, event: &KeyDownEvent, window, _| {
                if event.keystroke.key == "escape" {
                    window.remove_window();
                }
            }))
    }
}

fn wifi_device() -> String {
    let Ok(out) = Command::new("nmcli")
        .args(["-t", "-f", "DEVICE,TYPE", "device", "status"])
        .output()
    else {
        return "wlan0".into();
    };
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .find_map(|line| {
            let (dev, kind) = line.split_once(':')?;
            (kind == "wifi").then(|| dev.to_string())
        })
        .unwrap_or_else(|| "wlan0".into())
}

fn saved_ssids() -> Vec<String> {
    let Ok(out) = Command::new("nmcli")
        .args(["-t", "-f", "NAME,TYPE", "connection", "show"])
        .output()
    else {
        return Vec::new();
    };
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|line| {
            let (name, kind) = line.split_once(':')?;
            (kind == "802-11-wireless").then(|| name.to_string())
        })
        .collect()
}

fn scan(device: &str) -> Vec<Net> {
    let saved = saved_ssids();
    let Ok(out) = Command::new("nmcli")
        .args([
            "-t",
            "-f",
            "IN-USE,SSID,SIGNAL,SECURITY",
            "device",
            "wifi",
            "list",
            "ifname",
            device,
            "--rescan",
            "no",
        ])
        .output()
    else {
        return Vec::new();
    };
    let mut nets = Vec::new();
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let mut p = line.splitn(4, ':');
        let used = p.next().unwrap_or("");
        let ssid = p.next().unwrap_or("").to_string();
        if ssid.is_empty() || nets.iter().any(|n: &Net| n.ssid == ssid) {
            continue;
        }
        let signal = p.next().unwrap_or("0").parse().unwrap_or(0);
        let security = p.next().unwrap_or("").to_string();
        nets.push(Net {
            saved: saved.iter().any(|s| s == &ssid),
            active: used == "*",
            ssid,
            signal,
            security,
        });
    }
    nets.sort_by(|a, b| b.signal.cmp(&a.signal));
    nets
}

fn connect(ssid: &str, password: &str, use_saved: bool) -> String {
    let mut cmd = Command::new("nmcli");
    if use_saved {
        cmd.args(["connection", "up", "id", ssid]);
    } else if password.is_empty() {
        cmd.args(["device", "wifi", "connect", ssid]);
    } else {
        cmd.args(["device", "wifi", "connect", ssid, "password", password]);
    }
    match cmd.output() {
        Ok(out) if out.status.success() => "connected".into(),
        Ok(out) => String::from_utf8_lossy(&out.stderr).trim().to_string(),
        Err(e) => e.to_string(),
    }
}

fn disconnect(device: &str) -> String {
    match Command::new("nmcli")
        .args(["device", "disconnect", device])
        .output()
    {
        Ok(out) if out.status.success() => "disconnected".into(),
        Ok(out) => String::from_utf8_lossy(&out.stderr).trim().to_string(),
        Err(e) => e.to_string(),
    }
}

fn forget(ssid: &str) -> String {
    match Command::new("nmcli")
        .args(["connection", "delete", "id", ssid])
        .output()
    {
        Ok(out) if out.status.success() => "forgotten".into(),
        Ok(out) => String::from_utf8_lossy(&out.stderr).trim().to_string(),
        Err(e) => e.to_string(),
    }
}
