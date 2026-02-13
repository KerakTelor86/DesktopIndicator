use crate::config::{HotKey, Settings};
use crate::guard_clause;
use std::ffi::c_void;
use std::str::FromStr;
use std::thread;
use win_hotkeys::error::WHKError;
use win_hotkeys::{HotkeyManager, InterruptHandle, VKey};
use windows::Win32::Foundation::HWND;
use winvd::{get_desktops, move_window_to_desktop, switch_desktop};
use x_win::{get_active_window, WindowInfo};

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
    pub fn new(settings: &Settings) -> Result<Self, ShortcutError> {
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

        thread::spawn(move || {
            hkm.event_loop();
        });

        Ok(handler)
    }

    pub fn terminate(&self) {
        self.interrupt_handle.interrupt();
    }
}
