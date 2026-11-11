use purge::lifecycle::{Effect, Request, decide};

#[test]
fn native_and_client_window_controls_are_never_drawn_together() {
    use purge::lifecycle::custom_caption;
    use purge_engine::platform::OperatingSystem;
    assert!(custom_caption(OperatingSystem::Windows, false, false));
    assert!(custom_caption(OperatingSystem::Linux, false, true));
    assert!(!custom_caption(OperatingSystem::Linux, false, false));
    assert!(!custom_caption(OperatingSystem::MacOs, false, true));
    for system in [
        OperatingSystem::Windows,
        OperatingSystem::Linux,
        OperatingSystem::MacOs,
    ] {
        assert!(!custom_caption(system, true, true));
        assert!(!custom_caption(system, true, false));
    }
}

#[test]
fn missing_tray_or_disabled_preference_cannot_hide_the_only_window() {
    for (available, enabled) in [(false, false), (false, true), (true, false)] {
        assert_eq!(
            decide(Request::Minimize, available, enabled, true),
            Effect::MinimizeNormally
        );
    }
    assert_eq!(
        decide(Request::Minimize, true, true, true),
        Effect::HideToTray
    );
    assert_eq!(
        decide(Request::HideToTray, true, false, true),
        Effect::HideToTray
    );
    assert_eq!(
        decide(Request::HideToTray, false, true, true),
        Effect::MinimizeNormally
    );
}

#[test]
fn background_cleanup_can_be_restored_but_cannot_be_quit() {
    assert_eq!(
        decide(Request::Minimize, true, true, false),
        Effect::HideToTray
    );
    assert_eq!(decide(Request::Show, true, true, false), Effect::Show);
    assert_eq!(
        decide(Request::Quit, true, true, false),
        Effect::KeepRunning
    );
    assert_eq!(decide(Request::Quit, true, true, true), Effect::Quit);
}
