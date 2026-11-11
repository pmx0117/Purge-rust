/// Window lifecycle decisions are independent of the native tray backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Request {
    Show,
    Minimize,
    HideToTray,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    Show,
    HideToTray,
    MinimizeNormally,
    Quit,
    KeepRunning,
}

/// A compositor may ignore a request for client decorations. Draw controls
/// only for the decoration mode actually negotiated by the native window.
pub fn custom_caption(
    system: purge_engine::platform::OperatingSystem,
    fullscreen: bool,
    client_decorated: bool,
) -> bool {
    use purge_engine::platform::OperatingSystem;
    !fullscreen
        && (system == OperatingSystem::Windows
            || (system == OperatingSystem::Linux && client_decorated))
}

pub fn decide(
    request: Request,
    tray_available: bool,
    minimize_to_tray: bool,
    can_quit: bool,
) -> Effect {
    match request {
        Request::Show => Effect::Show,
        Request::Minimize | Request::HideToTray => {
            if tray_available && (minimize_to_tray || request == Request::HideToTray) {
                Effect::HideToTray
            } else {
                Effect::MinimizeNormally
            }
        }
        Request::Quit => {
            if can_quit {
                Effect::Quit
            } else {
                Effect::KeepRunning
            }
        }
    }
}
