use bus::Bus;
use std::sync::{Arc, Mutex};
use std::{sync, thread};
use winvd::{
    get_current_desktop, get_desktops, listen_desktop_events, Desktop, DesktopEvent as WinVdEvent,
};

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct DesktopInfo {
    pub name: String,
    pub index: u32,
}

impl From<Desktop> for DesktopInfo {
    fn from(desktop: Desktop) -> Self {
        Self {
            name: desktop.get_name().unwrap(),
            index: desktop.get_index().unwrap(),
        }
    }
}

const BUS_BUFFER_SIZE: usize = 32;

#[derive(Clone)]
pub struct DesktopEventHooks {
    on_active_change_hook: Arc<Mutex<Bus<Option<DesktopInfo>>>>,
    on_desktops_change_hook: Arc<Mutex<Bus<Option<Vec<DesktopInfo>>>>>,
}

impl DesktopEventHooks {
    pub fn new() -> Self {
        let (tx, rx) = sync::mpsc::channel::<WinVdEvent>();
        listen_desktop_events(tx).unwrap();

        let on_active_change_hook = Arc::new(Mutex::new(Bus::new(BUS_BUFFER_SIZE)));
        let on_desktops_change_hook = Arc::new(Mutex::new(Bus::new(BUS_BUFFER_SIZE)));

        let _thread = {
            let on_active_change_hook = on_active_change_hook.clone();
            let on_desktop_change_hook = on_desktops_change_hook.clone();

            thread::spawn(move || {
                for event in rx {
                    let current_desktop = get_current_desktop().unwrap();

                    if match event {
                        WinVdEvent::DesktopCreated(_) => true,
                        WinVdEvent::DesktopDestroyed { .. } => true,
                        WinVdEvent::DesktopChanged { .. } => true,
                        WinVdEvent::DesktopNameChanged(desktop, _) => desktop == current_desktop,
                        WinVdEvent::DesktopMoved { .. } => true,
                        _ => false,
                    } {
                        on_active_change_hook
                            .lock()
                            .unwrap()
                            .broadcast(Some(get_current_desktop().unwrap().into()));
                    }

                    if match event {
                        WinVdEvent::DesktopCreated(_) => true,
                        WinVdEvent::DesktopDestroyed { .. } => true,
                        WinVdEvent::DesktopNameChanged(_, _) => true,
                        WinVdEvent::DesktopMoved { .. } => true,
                        _ => false,
                    } {
                        on_desktop_change_hook.lock().unwrap().broadcast(Some(
                            get_desktops()
                                .unwrap()
                                .into_iter()
                                .map(|it| it.into())
                                .collect(),
                        ))
                    }
                }
            })
        };

        Self {
            on_active_change_hook,
            on_desktops_change_hook,
        }
    }

    pub fn on_active_desktop_change(&self, event_handler: impl Fn(DesktopInfo)) {
        event_handler(get_current_desktop().unwrap().into());

        let rx = self.on_active_change_hook.lock().unwrap().add_rx();
        for event in rx {
            if let Some(event) = event {
                event_handler(event);
            } else {
                break;
            }
        }
    }

    pub fn on_desktops_change(&self, event_handler: impl Fn(Vec<DesktopInfo>)) {
        event_handler(
            get_desktops()
                .unwrap()
                .into_iter()
                .map(|it| it.into())
                .collect(),
        );

        let rx = self.on_desktops_change_hook.lock().unwrap().add_rx();
        for event in rx {
            if let Some(event) = event {
                event_handler(event);
            } else {
                break;
            }
        }
    }

    pub fn terminate(&self) {
        self.on_active_change_hook.lock().unwrap().broadcast(None);
        self.on_desktops_change_hook.lock().unwrap().broadcast(None);
    }
}
