use crate::desktop::{DesktopEventHooks, DesktopInfo};
use crate::icon::IconSelector;
use std::thread;
use trayicon::{MenuBuilder, TrayIcon, TrayIconBuilder};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

#[derive(Clone, Eq, PartialEq, Debug)]
enum Event {
    ActiveDesktopChanged(DesktopInfo),
    LeftClick,
    DoubleClick,
    Exit,
}

pub struct TrayApp {
    window: Option<Window>,
    tray_icon: TrayIcon<Event>,
    icon_selector: IconSelector,
    desktop_event_hooks: DesktopEventHooks,
}

impl TrayApp {
    pub fn start(desktop_event_hooks: DesktopEventHooks) {
        let event_loop = EventLoop::<Event>::with_user_event().build().unwrap();
        event_loop.set_control_flow(ControlFlow::Wait);

        let proxy = event_loop.create_proxy();
        let tray_icon = TrayIconBuilder::new()
            .sender(move |event: &Event| {
                proxy.send_event(event.clone()).unwrap();
            })
            .tooltip("DesktopIndicator")
            .on_click(Event::LeftClick)
            .on_double_click(Event::DoubleClick)
            .menu(MenuBuilder::new().item("Exit", Event::Exit))
            .build()
            .unwrap();

        let mut app = TrayApp {
            window: None,
            tray_icon,
            icon_selector: IconSelector::new(desktop_event_hooks.clone()),
            desktop_event_hooks: desktop_event_hooks.clone(),
        };

        let _thread = {
            let proxy = event_loop.create_proxy();
            thread::spawn(move || {
                desktop_event_hooks.on_active_desktop_change(|info: DesktopInfo| {
                    proxy.send_event(Event::ActiveDesktopChanged(info)).unwrap();
                });
            })
        };

        event_loop.run_app(&mut app).unwrap();
    }
}

impl ApplicationHandler<Event> for TrayApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: Event) {
        match event {
            Event::ActiveDesktopChanged(info) => {
                if let Some(icon) = self.icon_selector.get_by_name(info.name) {
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
                event_loop.exit();
                self.desktop_event_hooks.terminate();
            }
            _ => {}
        }
    }
}
