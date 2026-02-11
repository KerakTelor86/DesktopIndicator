use crate::desktop::DesktopEventHooks;
use crate::tray::TrayApp;

mod desktop;
mod icon;
mod tray;

fn main() {
    TrayApp::start(DesktopEventHooks::new());
}
