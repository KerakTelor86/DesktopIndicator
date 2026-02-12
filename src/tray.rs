use crate::config::Settings;
use crate::desktop::{DesktopEventHooks, DesktopInfo};
use crate::guard_clause;
use crate::icon::IconSelector;
use std::thread;
use trayicon::{Error, MenuBuilder, TrayIcon, TrayIconBuilder};
use winit::application::ApplicationHandler;
use winit::error::EventLoopError;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::WindowId;

#[derive(Clone, Eq, PartialEq, Debug)]
enum Event {
    ActiveDesktopChanged(DesktopInfo),
    LeftClick,
    DoubleClick,
    Exit,
}

pub struct TrayApp {
    tray_icon: TrayIcon<Event>,
    icon_selector: IconSelector,
    desktop_event_hooks: DesktopEventHooks,
}

#[derive(Debug)]
pub enum TrayAppError {
    EventLoopError(EventLoopError),
    TrayIconBuildError(Error),
}

impl TrayApp {
    pub fn start(
        settings: Settings,
        desktop_event_hooks: DesktopEventHooks,
    ) -> Result<(), TrayAppError> {
        let event_loop = guard_clause!(EventLoop::<Event>::with_user_event().build(), error, {
            return Err(TrayAppError::EventLoopError(error));
        });
        event_loop.set_control_flow(ControlFlow::Wait);

        let proxy = event_loop.create_proxy();
        let tray_icon = guard_clause!(
            TrayIconBuilder::new()
                .sender(move |event: &Event| {
                    if let Err(error) = proxy.send_event(event.clone()) {
                        log::error!("Failed to send event from tray icon: {}", error);
                    }
                })
                .tooltip("DesktopIndicator")
                .on_click(Event::LeftClick)
                .on_double_click(Event::DoubleClick)
                .menu(MenuBuilder::new().item("Exit", Event::Exit))
                .build(),
            error,
            {
                return Err(TrayAppError::TrayIconBuildError(error));
            }
        );

        let mut app = TrayApp {
            tray_icon,
            icon_selector: IconSelector::new(desktop_event_hooks.clone()),
            desktop_event_hooks: desktop_event_hooks.clone(),
        };

        let _thread = {
            let proxy = event_loop.create_proxy();
            thread::spawn(move || {
                desktop_event_hooks.on_active_desktop_change(|info: DesktopInfo| {
                    if let Err(_) = proxy.send_event(Event::ActiveDesktopChanged(info)) {
                        return;
                    }
                });
            })
        };

        if let Err(error) = event_loop.run_app(&mut app) {
            return Err(TrayAppError::EventLoopError(error));
        };
        Ok(())
    }
}

impl ApplicationHandler<Event> for TrayApp {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: Event) {
        match event {
            Event::ActiveDesktopChanged(info) => {
                if let Some(icon) = self.icon_selector.get_by_name(&info.name) {
                    self.tray_icon.set_icon(icon.as_ref()).unwrap();
                } else if let Some(icon) = self.icon_selector.get_by_index(info.index) {
                    self.tray_icon.set_icon(icon.as_ref()).unwrap();
                } else {
                    self.tray_icon
                        .set_icon(self.icon_selector.get_default().as_ref())
                        .unwrap();
                }
            }
            Event::LeftClick => {
                todo!();
            }
            Event::DoubleClick => {
                todo!();
            }
            Event::Exit => {
                self.desktop_event_hooks.terminate();
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                self.desktop_event_hooks.terminate();
                event_loop.exit();
            }
            _ => {}
        }
    }
}
