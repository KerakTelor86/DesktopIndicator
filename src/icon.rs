use crate::desktop::{DesktopEventHooks, DesktopInfo};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use trayicon::Icon;

#[derive(Clone, Debug)]
pub struct IconSelector {
    icons: Arc<Mutex<Vec<Arc<Icon>>>>,
    name_to_index: Arc<Mutex<HashMap<String, u32>>>,
}

impl IconSelector {
    pub fn new(desktop_event_hooks: DesktopEventHooks) -> Self {
        let selector = Self {
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
        if let Some(icon) = self.icons.lock().unwrap().get(index as usize) {
            Some(icon.clone())
        } else {
            None
        }
    }

    pub fn get_by_name(&self, name: String) -> Option<Arc<Icon>> {
        if let Some(index) = self.name_to_index.lock().unwrap().get(&name) {
            self.get_by_index(index.clone())
        } else {
            None
        }
    }

    pub fn get_default(&self) -> Arc<Icon> {
        todo!();
    }

    fn build_icons(&self, desktops: Vec<DesktopInfo>) {
        todo!();
    }
}
