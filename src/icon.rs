use crate::config::Settings;
use crate::desktop::{DesktopEventHooks, DesktopInfo};
use crate::guard_clause;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::{fs, thread};
use trayicon::Icon;

#[derive(Clone, Debug)]
pub struct IconSelector {
    settings: Arc<Settings>,
    default_icon: Option<Arc<Icon>>,
    icons: Arc<Mutex<Vec<Arc<Icon>>>>,
    name_to_index: Arc<Mutex<HashMap<String, u32>>>,
}

impl IconSelector {
    pub fn new(settings: Settings, desktop_event_hooks: DesktopEventHooks) -> Self {
        let settings = Arc::new(settings);
        let default_icon = load_icon(&settings.default_icon_path);
        let selector = Self {
            settings,
            default_icon,
            icons: Arc::new(Mutex::new(vec![])),
            name_to_index: Arc::new(Mutex::new(HashMap::new())),
        };

        let _thread = {
            let selector = selector.clone();
            thread::spawn(move || {
                desktop_event_hooks.on_desktops_change(|desktops: Vec<DesktopInfo>| {
                    selector.build_icons(desktops);
                })
            })
        };

        selector
    }

    pub fn get_by_index(&self, index: u32) -> Option<Arc<Icon>> {
        let icons = guard_clause!(self.icons.lock(), {
            return None;
        });
        if let Some(icon) = icons.get(index as usize) {
            Some(icon.clone())
        } else {
            None
        }
    }

    pub fn get_by_name(&self, name: &str) -> Option<Arc<Icon>> {
        let name_to_index = guard_clause!(self.name_to_index.lock(), {
            return None;
        });
        if let Some(index) = name_to_index.get(name) {
            self.get_by_index(index.clone())
        } else {
            None
        }
    }

    pub fn get_default(&self) -> Option<Arc<Icon>> {
        self.default_icon.clone()
    }

    fn build_icons(&self, desktops: Vec<DesktopInfo>) {
        let mut icons = guard_clause!(self.icons.lock(), {
            return;
        });
        let mut name_to_index = guard_clause!(self.name_to_index.lock(), {
            return;
        });

        icons.clear();
        name_to_index.clear();

        let name_to_path = &self.settings.desktop_name_to_icon_path;

        for desktop in &desktops {
            let icon = name_to_path
                .get(&desktop.name)
                .and_then(|path| load_icon(path));

            if let Some(icon) = icon {
                name_to_index.insert(desktop.name.clone(), desktop.index);
                while icons.len() <= desktop.index as usize {
                    icons.push(self.get_default().unwrap_or(icon.clone()));
                }
                icons[desktop.index as usize] = icon;
            }
        }
    }
}

fn load_icon(path: &str) -> Option<Arc<Icon>> {
    let buffer = guard_clause!(fs::read(path), error, {
        log::error!("Failed to read icon file '{}': {}", path, error);
        return None;
    });
    let buffer: &'static [u8] = Box::leak(buffer.into_boxed_slice());
    let icon = guard_clause!(Icon::from_buffer(buffer, None, None), error, {
        log::error!("Failed to load icon from '{}': {}", path, error);
        return None;
    });
    Some(Arc::new(icon))
}
