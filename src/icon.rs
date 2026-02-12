use crate::config::Settings;
use crate::guard_clause;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use trayicon::Icon;

#[derive(Clone, Debug)]
pub struct IconSelector {
    default_icon: Option<Arc<Icon>>,
    index_to_icon: Arc<HashMap<u32, Option<Arc<Icon>>>>,
    name_to_icon: Arc<HashMap<String, Option<Arc<Icon>>>>,
}

impl IconSelector {
    pub fn new(settings: Settings) -> Self {
        let default_icon = load_icon(&settings.default_icon_path);
        let index_to_icon = Arc::new(
            settings
                .desktop_index_to_icon_path
                .into_iter()
                .map(|(index, path)| (index, load_icon(&path)))
                .collect::<HashMap<_, _>>(),
        );
        let name_to_icon = Arc::new(
            settings
                .desktop_name_to_icon_path
                .into_iter()
                .map(|(name, path)| (name, load_icon(&path)))
                .collect::<HashMap<_, _>>(),
        );

        let selector = Self {
            default_icon,
            index_to_icon,
            name_to_icon,
        };

        selector
    }

    pub fn get_by_index(&self, index: u32) -> Option<Arc<Icon>> {
        self.index_to_icon.get(&index)?.clone()
    }

    pub fn get_by_name(&self, name: &str) -> Option<Arc<Icon>> {
        self.name_to_icon.get(name)?.clone()
    }

    pub fn get_default(&self) -> Option<Arc<Icon>> {
        self.default_icon.clone()
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
