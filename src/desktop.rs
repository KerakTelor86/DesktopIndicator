use crate::guard_clause;
use bus::Bus;
use std::sync::{Arc, Mutex};
use std::{sync, thread};
use winvd::{
    get_current_desktop, get_desktops, listen_desktop_events, Desktop, DesktopEvent, DesktopEventThread,
    Error,
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
    _listener_thread: Arc<DesktopEventThread>,
    on_active_change_hook: Arc<Mutex<Bus<Option<DesktopInfo>>>>,
    on_desktops_change_hook: Arc<Mutex<Bus<Option<Vec<DesktopInfo>>>>>,
}

#[allow(unused)]
impl DesktopEventHooks {
    pub fn new() -> Result<Self, Error> {
        let (tx, rx) = sync::mpsc::channel::<DesktopEvent>();
        let listener_thread = listen_desktop_events(tx)?;

        let on_active_change_hook = Arc::new(Mutex::new(Bus::new(BUS_BUFFER_SIZE)));
        let on_desktops_change_hook = Arc::new(Mutex::new(Bus::new(BUS_BUFFER_SIZE)));

        let _thread = {
            let on_active_change_hook = on_active_change_hook.clone();
            let on_desktops_change_hook = on_desktops_change_hook.clone();

            thread::spawn(move || {
                for event in rx {
                    log::info!("Event received: {:?}", event);
                    let current_desktop = guard_clause!(get_current_desktop(), error, {
                        log::error!("Could not get current desktop: {:?}", error);
                        continue;
                    });

                    if match event {
                        DesktopEvent::DesktopCreated(_) => true,
                        DesktopEvent::DesktopDestroyed { .. } => true,
                        DesktopEvent::DesktopChanged { .. } => true,
                        DesktopEvent::DesktopNameChanged(desktop, _) => desktop == current_desktop,
                        DesktopEvent::DesktopMoved { .. } => true,
                        _ => false,
                    } {
                        let Ok(mut locked_hook) = on_active_change_hook.try_lock() else {
                            log::error!("Could not lock the active desktop change hook");
                            continue;
                        };
                        locked_hook.broadcast(Some(current_desktop.into()));
                    }

                    if match event {
                        DesktopEvent::DesktopCreated(_) => true,
                        DesktopEvent::DesktopDestroyed { .. } => true,
                        DesktopEvent::DesktopNameChanged(_, _) => true,
                        DesktopEvent::DesktopMoved { .. } => true,
                        _ => false,
                    } {
                        let Ok(mut locked_hook) = on_desktops_change_hook.try_lock() else {
                            log::error!("Could not lock the desktop change hook");
                            continue;
                        };
                        let desktops = guard_clause!(get_desktops(), error, {
                            log::error!("Could not get desktops: {:?}", error);
                            continue;
                        });
                        locked_hook
                            .broadcast(Some(desktops.into_iter().map(|it| it.into()).collect()));
                    }
                }
            })
        };

        Ok(Self {
            _listener_thread: Arc::new(listener_thread),
            on_active_change_hook,
            on_desktops_change_hook,
        })
    }

    pub fn on_active_desktop_change(&self, event_handler: impl Fn(DesktopInfo)) {
        let current_desktop = guard_clause!(get_current_desktop(), error, {
            log::error!("Could not get current desktop: {:?}", error);
            return;
        });
        event_handler(current_desktop.into());

        let mut change_hook = guard_clause!(self.on_active_change_hook.try_lock(), error, {
            log::error!("Could not lock the active desktop change hook: {:?}", error);
            return;
        });
        let rx = change_hook.add_rx();
        drop(change_hook);

        for event in rx {
            if let Some(event) = event {
                event_handler(event);
            } else {
                break;
            }
        }
    }

    pub fn on_desktops_change(&self, event_handler: impl Fn(Vec<DesktopInfo>)) {
        let desktops = guard_clause!(get_desktops(), error, {
            log::error!("Could not get desktops: {:?}", error);
            return;
        });
        event_handler(desktops.into_iter().map(|it| it.into()).collect());

        let mut change_hook = guard_clause!(self.on_desktops_change_hook.try_lock(), error, {
            log::error!("Could not lock the desktops change hook: {:?}", error);
            return;
        });
        let rx = change_hook.add_rx();
        drop(change_hook);

        for event in rx {
            if let Some(event) = event {
                event_handler(event);
            } else {
                break;
            }
        }
    }

    pub fn terminate(&self) {
        if let Ok(mut hook) = self.on_active_change_hook.lock() {
            hook.broadcast(None);
        }
        if let Ok(mut hook) = self.on_desktops_change_hook.lock() {
            hook.broadcast(None);
        }
    }
}
