use super::*;
pub fn run() -> Result<()> {
    Application::new()
        .with_quit_mode(QuitMode::Explicit)
        .with_assets(assets::Assets)
        .run(|cx: &mut App| {
            #[cfg(target_os = "macos")]
            purge::macos_window::set_application_icon(include_bytes!(concat!(
                env!("OUT_DIR"),
                "/purge.png"
            )));
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
                KeyBinding::new("ctrl-m", MinimizeToTray, None),
            ]);
            #[cfg(target_os = "macos")]
            cx.bind_keys([
                KeyBinding::new("cmd-r", Rescan, None),
                KeyBinding::new("cmd-shift-r", ScanAll, None),
                KeyBinding::new("cmd-f", Search, None),
                KeyBinding::new("cmd-1", Overview, None),
                KeyBinding::new("cmd-2", Caches, None),
                KeyBinding::new("cmd-3", Developer, None),
                KeyBinding::new("cmd-4", LargeFiles, None),
                KeyBinding::new("cmd-5", Applications, None),
                KeyBinding::new("cmd-m", MinimizeToTray, None),
                KeyBinding::new("cmd-q", QuitApplication, None),
            ]);
            cx.on_action(|_: &MinimizeToTray, cx| {
                shell::request(purge::lifecycle::Request::Minimize, cx)
            });
            cx.on_action(|_: &QuitApplication, cx| {
                shell::request(purge::lifecycle::Request::Quit, cx)
            });
            let view = cx.new(PurgeView::new);
            shell::Shell::install(view, cx);
            cx.activate(true);
        });
    Ok(())
}
