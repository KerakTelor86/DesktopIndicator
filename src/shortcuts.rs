use crate::config::{HotKey, Settings};
use crate::desktop::DesktopEventHooks;
use crate::guard_clause;
use std::ffi::c_void;
use std::str::FromStr;
use std::thread;
use win_hotkeys::error::WHKError;
use win_hotkeys::{HotkeyManager, InterruptHandle, VKey};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow;
use winvd::{get_desktops, is_window_on_current_desktop, move_window_to_desktop, switch_desktop};
use x_win::{get_active_window, get_open_windows, WindowInfo};

pub struct ShortcutHandler {
    interrupt_handle: InterruptHandle,
}

#[derive(Debug)]
#[allow(unused)]
pub enum ShortcutError {
    InvalidKey(WHKError),
    HotKeyRegistrationFailed(WHKError),
}

impl HotKey {
    fn parse(&self) -> Result<(VKey, Vec<VKey>, u32), WHKError> {
        let HotKey {
            modifier_keys,
            trigger_key,
            target_desktop_index,
        } = self;
        let trigger_key = guard_clause!(VKey::from_str(&trigger_key), error, {
            return Err(error);
        });

        let modifier_keys: Result<Vec<_>, _> =
            modifier_keys.iter().map(|it| VKey::from_str(&it)).collect();

        let modifier_keys = guard_clause!(modifier_keys, error, {
            return Err(error);
        });

        Ok((trigger_key, modifier_keys, *target_desktop_index))
    }
}

impl ShortcutHandler {
    pub fn new(
        settings: &Settings,
        desktop_event_hooks: DesktopEventHooks,
    ) -> Result<Self, ShortcutError> {
        let mut hkm = HotkeyManager::new();

        let handler = Self {
            interrupt_handle: hkm.interrupt_handle(),
        };

        for hotkey in &settings.switch_desktop_hotkeys {
            let (trigger_key, modifier_keys, desktop_index) =
                guard_clause!(hotkey.parse(), error, {
                    return Err(ShortcutError::InvalidKey(error));
                });

            let target_index = desktop_index.clone() as usize;
            let switch_lambda = move || {
                let current_desktops = guard_clause!(get_desktops(), error, {
                    log::error!("Failed to get desktops: {:?}", error);
                    return;
                });

                match current_desktops.get(target_index) {
                    None => log::error!(
                        "Desktop index not found while attempting to switch: {}",
                        target_index,
                    ),
                    Some(&target_desktop) => {
                        if let Err(error) = switch_desktop(target_desktop) {
                            log::error!("Failed to switch desktop: {:?}", error);
                        }
                    }
                }
            };

            if let Err(error) = hkm.register_hotkey(trigger_key, &modifier_keys, switch_lambda) {
                return Err(ShortcutError::HotKeyRegistrationFailed(error));
            }
        }

        for hotkey in &settings.move_window_hotkeys {
            let (trigger_key, modifier_keys, desktop_index) =
                guard_clause!(hotkey.parse(), error, {
                    return Err(ShortcutError::InvalidKey(error));
                });

            let target_index = desktop_index.clone() as usize;
            let follow_moved_windows = settings.follow_moved_windows.clone();

            let switch_lambda = move || {
                let WindowInfo {
                    id: target_hwnd, ..
                } = guard_clause!(get_active_window(), error, {
                    log::error!("Failed to get current active window: {:?}", error);
                    return;
                });

                let target_window = HWND(target_hwnd as *mut c_void);

                let current_desktops = guard_clause!(get_desktops(), error, {
                    log::error!("Failed to get desktops: {:?}", error);
                    return;
                });

                match current_desktops.get(target_index) {
                    None => log::error!(
                        "Desktop index not found while attempting to move window: {}",
                        target_index,
                    ),
                    Some(&target_desktop) => {
                        if let Err(error) = move_window_to_desktop(target_desktop, &target_window) {
                            log::error!("Failed to switch desktop: {:?}", error);
                        }
                        if !follow_moved_windows {
                            return;
                        }
                        if let Err(error) = switch_desktop(target_desktop) {
                            log::error!("Failed to switch desktop: {:?}", error);
                        }
                    }
                }
            };

            if let Err(error) = hkm.register_hotkey(trigger_key, &modifier_keys, switch_lambda) {
                return Err(ShortcutError::HotKeyRegistrationFailed(error));
            }
        }

        // Focus first window on desktop change to fix wrong input focus after desktop switch
        thread::spawn(move || {
            desktop_event_hooks.on_active_desktop_change(|_| {
                let open_windows = guard_clause!(get_open_windows(), error, {
                    log::error!("Failed to get open windows: {:?}", error);
                    return;
                });

                let target_window = {
                    let window_on_desktop = open_windows.into_iter().find(|window| {
                        // Minimized windows have negative coordinates
                        // Full screen windows also have negative coordinates (presumably padding?)
                        if !window.position.is_full_screen
                            && (window.position.x <= 0 || window.position.y <= 0)
                        {
                            return false;
                        }
                        let window_handle = HWND(window.id as *mut c_void);
                        is_window_on_current_desktop(window_handle).unwrap_or(false)
                    });
                    let Some(target_window) = window_on_desktop else {
                        // Expected - Desktop probably has no open windows
                        return;
                    };
                    target_window
                };

                // Weird calling semantics...
                if unsafe { SetForegroundWindow(HWND(target_window.id as *mut c_void)).0 } == 0 {
                    log::error!("Failed to set active window");
                    return;
                }

                log::info!("Set active window: {:?}", target_window);
            })
        });

        thread::spawn(move || {
            hkm.event_loop();
        });

        Ok(handler)
    }

    pub fn terminate(&self) {
        self.interrupt_handle.interrupt();
    }
}
