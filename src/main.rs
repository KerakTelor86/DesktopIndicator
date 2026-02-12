use crate::config::Settings;
use crate::desktop::DesktopEventHooks;
use crate::tray::TrayApp;
use simple_logger::SimpleLogger;

mod config;
mod desktop;
mod icon;
mod tray;
mod utils;

fn main() {
    if let Err(error) = SimpleLogger::new().init() {
        eprintln!("Failed to initialize logger: {}", error);
        return;
    }

    let settings = guard_clause!(Settings::new(), error, {
        log::error!("Error while reading settings: {:?}", error);
        return;
    });

    let desktop_event_hooks = guard_clause!(DesktopEventHooks::new(), error, {
        log::error!("Error initializing desktop event hooks: {:?}", error);
        return;
    });

    if let Err(error) = TrayApp::start(settings, desktop_event_hooks) {
        log::error!("Error from TrayApp: {:?}", error)
    }
}
