use crate::config::Settings;
use crate::desktop::{DesktopEventHooks, DesktopInfo};
use crate::guard_clause;
use crate::icon::IconSelector;
use std::{process, thread};
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
    Exit,
}

pub struct TrayApp {
    tray_icon: TrayIcon<Event>,
    icon_selector: IconSelector,
    desktop_event_hooks: DesktopEventHooks,
}

#[derive(Debug)]
#[allow(unused)]
pub enum TrayAppError {
    EventLoopError(EventLoopError),
    TrayIconBuildError(Error),
    MissingDefaultIcon,
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
        let icon_selector = IconSelector::new(settings);

        let Some(default_icon) = icon_selector.get_default() else {
            return Err(TrayAppError::MissingDefaultIcon);
        };

        let tray_icon = guard_clause!(
            TrayIconBuilder::new()
                .sender(move |event: &Event| {
                    if let Err(error) = proxy.send_event(event.clone()) {
                        log::error!("Failed to send event from tray icon: {}", error);
                    }
                })
                .icon(default_icon.as_ref().clone())
                .tooltip("DesktopIndicator")
                .on_click(Event::LeftClick)
                .menu(MenuBuilder::new().item("Exit", Event::Exit))
                .build(),
            error,
            {
                return Err(TrayAppError::TrayIconBuildError(error));
            }
        );

        let mut app = TrayApp {
            tray_icon,
            icon_selector,
            desktop_event_hooks: desktop_event_hooks.clone(),
        };

        let _thread = {
            let proxy = event_loop.create_proxy();
            thread::spawn(move || {
                desktop_event_hooks.on_active_desktop_change(|info: DesktopInfo| {
                    log::info!("{:?}", info);
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
                let Some(icon) = self
                    .icon_selector
                    .get_by_name(&info.name)
                    .or(self.icon_selector.get_by_index(info.index))
                    .or(self.icon_selector.get_default())
                else {
                    log::error!("Failed to select tray icon (Perhaps no default was set?)");
                    return;
                };

                if let Err(error) = self.tray_icon.set_icon(icon.as_ref()) {
                    log::error!("Failed to set tray icon: {}", error);
                }
            }
            Event::LeftClick => {
                // https://stackoverflow.com/a/79009385/10661599
                if let Err(error) = process::Command::new("explorer")
                    .arg("shell:::{3080F90E-D7AD-11D9-BD98-0000947B0257}")
                    .spawn()
                {
                    log::error!("Could not open task view: {}", error);
                };
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
