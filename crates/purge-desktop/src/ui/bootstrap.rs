use super::*;
pub fn run() -> Result<()> {
    Application::new()
        .with_assets(assets::Assets)
        .run(|cx: &mut App| {
            if let Err(error) = cx
                .text_system()
                .add_fonts(vec![Cow::Borrowed(include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../assets/fonts/OnestText.ttf"
                )))])
            {
                eprintln!("Onest: {error}");
            }
            input::bind_keys(cx);
            cx.bind_keys([
                KeyBinding::new("ctrl-r", Rescan, None),
                KeyBinding::new("ctrl-shift-r", ScanAll, None),
                KeyBinding::new("ctrl-f", Search, None),
                KeyBinding::new("ctrl-1", Overview, None),
                KeyBinding::new("ctrl-2", Caches, None),
                KeyBinding::new("ctrl-3", Developer, None),
                KeyBinding::new("ctrl-4", LargeFiles, None),
                KeyBinding::new("ctrl-5", Applications, None),
                KeyBinding::new("escape", Escape, None),
                KeyBinding::new("ctrl-alt-1", AllItems, None),
                KeyBinding::new("ctrl-alt-2", SafeItems, None),
                KeyBinding::new("ctrl-alt-3", ReviewItems, None),
                KeyBinding::new("tab", NextControl, None),
                KeyBinding::new("shift-tab", PreviousControl, None),
            ]);
            cx.on_window_closed(|cx| {
                if cx.windows().is_empty() {
                    cx.quit();
                }
            })
            .detach();
            let bounds = Bounds::centered(None, size(px(980.), px(700.)), cx);
            if let Err(e) = cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    window_min_size: Some(size(px(900.), px(600.))),
                    titlebar: Some(TitlebarOptions {
                        title: Some("Purge · Windows".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |w, cx| {
                    let app = cx.new(|cx| {
                        let app = PurgeView::new(cx);
                        app.focus.focus(w);
                        app
                    });
                    let weak = app.downgrade();
                    w.on_window_should_close(cx, move |_, cx| {
                        weak.update(cx, |app, cx| {
                            if !app.controller.can_close() {
                                app.controller.model.notice =
                                    Some("正在移入回收站，请等待本次操作完成后再关闭。".into());
                                cx.notify();
                                false
                            } else {
                                true
                            }
                        })
                        .unwrap_or(true)
                    });
                    app
                },
            ) {
                eprintln!("无法打开窗口：{e:#}");
                cx.quit();
            }
            cx.activate(true);
        });
    Ok(())
}
