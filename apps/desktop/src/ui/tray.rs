use anyhow::Result;
use purge::lifecycle::Request;
use std::sync::mpsc::{self, Receiver};
use tray_icon::{
    MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};

pub(super) struct SystemTray {
    _icon: TrayIcon,
    events: Receiver<Request>,
    quit: MenuItem,
    can_quit: std::cell::Cell<bool>,
}

impl SystemTray {
    pub fn new() -> Result<Self> {
        #[cfg(target_os = "linux")]
        if !host_available() {
            anyhow::bail!("当前桌面未提供系统托盘，将使用普通最小化。");
        }
        let menu = Menu::new();
        let show = MenuItem::with_id("purge-show", "显示 Purge", true, None);
        let hide = MenuItem::with_id("purge-hide", "最小化到托盘", true, None);
        let quit = MenuItem::with_id("purge-quit", "退出 Purge", true, None);
        menu.append_items(&[&show, &hide, &PredefinedMenuItem::separator(), &quit])?;
        let image =
            image::load_from_memory(include_bytes!(concat!(env!("OUT_DIR"), "/purge-32.png")))?
                .into_rgba8();
        let icon = tray_icon::Icon::from_rgba(image.into_raw(), 32, 32)?;
        let icon = TrayIconBuilder::new()
            .with_id("purge")
            .with_tooltip("Purge")
            .with_icon(icon)
            .with_menu(Box::new(menu))
            .with_menu_on_left_click(false)
            .build()?;
        let (sender, events) = mpsc::channel();
        let menu_sender = sender.clone();
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            let action = match event.id.as_ref() {
                "purge-show" => Request::Show,
                "purge-hide" => Request::HideToTray,
                "purge-quit" => Request::Quit,
                _ => return,
            };
            let _ = menu_sender.send(action);
        }));
        TrayIconEvent::set_event_handler(Some(move |event| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } | TrayIconEvent::DoubleClick {
                    button: MouseButton::Left,
                    ..
                }
            ) {
                let _ = sender.send(Request::Show);
            }
        }));
        Ok(Self {
            _icon: icon,
            events,
            quit,
            can_quit: std::cell::Cell::new(true),
        })
    }

    pub fn poll(&self) -> Option<Request> {
        self.events.try_recv().ok()
    }
    pub fn set_can_quit(&self, can_quit: bool) {
        if self.can_quit.replace(can_quit) != can_quit {
            self.quit.set_enabled(can_quit);
        }
    }
    pub fn available(&self) -> bool {
        #[cfg(target_os = "linux")]
        {
            host_available()
        }
        #[cfg(not(target_os = "linux"))]
        {
            true
        }
    }
}

#[cfg(target_os = "linux")]
fn host_available() -> bool {
    let Ok(connection) = zbus::blocking::Connection::session() else {
        return false;
    };
    let Ok(proxy) = zbus::blocking::Proxy::new(
        &connection,
        "org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        "org.freedesktop.DBus",
    ) else {
        return false;
    };
    proxy
        .call::<_, _, bool>("NameHasOwner", &("org.kde.StatusNotifierWatcher",))
        .unwrap_or(false)
}
