use super::*;
use purge::lifecycle::{self, Effect, Request};
use std::time::Instant;

pub(super) struct Shell {
    view: Entity<PurgeView>,
    window: Option<WindowHandle<PurgeView>>,
    tray: Option<tray::SystemTray>,
    hidden: bool,
    host_checked: Instant,
}
impl Global for Shell {}

impl Shell {
    pub fn install(view: Entity<PurgeView>, cx: &mut App) {
        let tray = match tray::SystemTray::new() {
            Ok(tray) => Some(tray),
            Err(error) => {
                view.update(cx, |view, _| {
                    view.controller.model.notice = Some(format!("托盘不可用：{error}"))
                });
                None
            }
        };
        cx.set_global(Self {
            view,
            window: None,
            tray,
            hidden: false,
            host_checked: Instant::now(),
        });
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() && !cx.global::<Shell>().hidden {
                cx.quit();
            }
        })
        .detach();
        cx.spawn(async |cx| {
            loop {
                smol::Timer::after(Duration::from_millis(120)).await;
                cx.update(Self::poll);
            }
        })
        .detach();
        Self::handle(Request::Show, cx);
    }

    fn poll(cx: &mut App) {
        let window = cx.global::<Shell>().window;
        let native_minimized = window
            .and_then(|handle| {
                handle
                    .update(cx, |_, window, _| platform::is_minimized(window))
                    .ok()
            })
            .unwrap_or(false);
        let (event, restore, native_minimized) = {
            let shell = cx.global::<Shell>();
            let event = shell.tray.as_ref().and_then(|tray| tray.poll());
            let restore = shell.hidden && shell.host_checked.elapsed() > Duration::from_secs(2);
            (
                event,
                restore,
                native_minimized
                    && shell
                        .view
                        .read(cx)
                        .controller
                        .model
                        .settings
                        .minimize_to_tray
                    && shell.tray.is_some(),
            )
        };
        if let Some(event) = event {
            Self::handle(event, cx);
        }
        if native_minimized {
            Self::handle(Request::Minimize, cx);
        }
        if restore {
            cx.global_mut::<Shell>().host_checked = Instant::now();
            if !Self::tray_available(cx) {
                Self::handle(Request::Show, cx);
            }
        }
        let shell = cx.global::<Shell>();
        if let Some(tray) = &shell.tray {
            tray.set_can_quit(shell.view.read(cx).controller.can_close());
        }
    }

    pub fn tray_available(cx: &App) -> bool {
        cx.try_global::<Shell>()
            .and_then(|shell| shell.tray.as_ref())
            .is_some_and(|tray| tray.available())
    }

    pub fn handle(request: Request, cx: &mut App) {
        let view = cx.global::<Shell>().view.clone();
        let model = view.read(cx);
        let effect = lifecycle::decide(
            request,
            Self::tray_available(cx),
            model.controller.model.settings.minimize_to_tray,
            model.controller.can_close(),
        );
        match effect {
            Effect::Show => Self::show(cx),
            Effect::HideToTray => {
                let shell = cx.global_mut::<Shell>();
                shell.hidden = true;
                if let Some(handle) = shell.window.take() {
                    let _ = handle.update(cx, |_, window, _| window.remove_window());
                }
            }
            Effect::MinimizeNormally => {
                if let Some(handle) = cx.global::<Shell>().window {
                    let _ = handle.update(cx, |_, window, _| window.minimize_window());
                }
            }
            Effect::Quit => cx.quit(),
            Effect::KeepRunning => {
                view.update(cx, |view, cx| {
                    view.controller.model.notice =
                        Some("正在移入回收站，请等待本次操作完成后再退出。".into());
                    cx.notify();
                });
                Self::show(cx);
            }
        }
    }

    fn show(cx: &mut App) {
        if let Some(handle) = cx.global::<Shell>().window {
            let _ = handle.update(cx, |_, window, _| window.activate_window());
            cx.activate(true);
            return;
        }
        let view = cx.global::<Shell>().view.clone();
        let bounds = Bounds::centered(None, size(px(980.), px(700.)), cx);
        let result = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(900.), px(600.))),
                titlebar: Some(TitlebarOptions {
                    title: Some("Purge".into()),
                    appears_transparent: !cfg!(target_os = "macos"),
                    ..Default::default()
                }),
                app_id: Some("dev.purge.Purge".into()),
                window_decorations: Some(WindowDecorations::Client),
                ..Default::default()
            },
            move |window, cx| {
                // GPUI's Wayland constructor does not apply TitlebarOptions.title.
                window.set_window_title("Purge");
                let weak = view.downgrade();
                window.on_window_should_close(cx, move |_, cx| {
                    weak.update(cx, |view, cx| {
                        if !view.controller.can_close() {
                            view.controller.model.notice =
                                Some("正在移入回收站，请等待本次操作完成后再关闭。".into());
                            cx.notify();
                            false
                        } else {
                            true
                        }
                    })
                    .unwrap_or(true)
                });
                view.update(cx, |view, cx| {
                    view.focus.focus(window, cx);
                    view.native_dark = None;
                    #[cfg(windows)]
                    {
                        view.native_icon_scale = None;
                    }
                    cx.notify();
                });
                view
            },
        );
        match result {
            Ok(handle) => {
                let shell = cx.global_mut::<Shell>();
                shell.window = Some(handle);
                shell.hidden = false;
                cx.activate(true);
            }
            Err(error) => {
                eprintln!("无法打开窗口：{error:#}");
                if !Self::tray_available(cx) {
                    cx.quit();
                }
            }
        }
    }
}

pub(super) fn request(action: Request, cx: &mut App) {
    // View callbacks run with their entity borrowed; defer before reading the
    // controller again, including tray menu actions and window-close guards.
    cx.defer(move |cx| Shell::handle(action, cx));
}
