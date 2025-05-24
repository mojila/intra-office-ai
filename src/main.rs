use objc2_core_foundation::CFRunLoop;
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    TrayIcon, TrayIconBuilder,
};
use winit::{
    application::ApplicationHandler,
    event_loop::EventLoop,
};

const APP_TITLE: &str = "Intra Office AI";

#[derive(Debug)]
enum UserEvent {
    MenuEvent(tray_icon::menu::MenuEvent),
}

struct TrayIconMenuItem {
    title: MenuItem,
    quit: MenuItem,
}

impl TrayIconMenuItem  {
    fn new() -> TrayIconMenuItem {
        let title: MenuItem = MenuItem::new(APP_TITLE, false, None);
        let quit: MenuItem = MenuItem::new("Quit", true, None);

        TrayIconMenuItem { title, quit }
    }
}

struct Application {
    tray_icon: Option<TrayIcon>,
    menu_item: TrayIconMenuItem,
}

impl Application {
    fn new() -> Application {
        let menu_item: TrayIconMenuItem = TrayIconMenuItem::new();

        Application { tray_icon: None, menu_item }
    }

    fn new_tray_icon(&self) -> TrayIcon {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/asset/icon.png");
        let icon = load_icon(std::path::Path::new(path));

        TrayIconBuilder::new()
            .with_menu(Box::new(self.new_tray_menu()))
            .with_tooltip(APP_TITLE.to_string())
            .with_icon(icon)
            .build()
            .unwrap()
    }

    fn new_tray_menu(&self) -> Menu {
        let menu: Menu = Menu::new();

        if let Err(err) = menu.append(&self.menu_item.title) {
            println!("{err:?}");
        }
        
        if let Err(err) = menu.append(&self.menu_item.quit) {
            println!("{err:?}");
        }

        menu
    }
}

impl ApplicationHandler<UserEvent> for Application {
    fn resumed(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {}

    fn window_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        _event: winit::event::WindowEvent,
    ) {
    }

    fn new_events(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        cause: winit::event::StartCause,
    ) {
        // We create the icon once the event loop is actually running
        // to prevent issues like https://github.com/tauri-apps/tray-icon/issues/90
        if winit::event::StartCause::Init == cause {
            #[cfg(not(target_os = "linux"))]
            {
                self.tray_icon = Some(self.new_tray_icon());
            }

            // We have to request a redraw here to have the icon actually show up.
            // Winit only exposes a redraw method on the Window so we use core-foundation directly.
            #[cfg(target_os = "macos")]
            {
                let rl = CFRunLoop::main().unwrap();
                CFRunLoop::wake_up(&rl);
            }
        }
    }

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::MenuEvent(menu_event) => {
                match menu_event {
                    MenuEvent { id } => {
                        let quit_menu_id = self.menu_item.quit.id().clone();

                        if quit_menu_id == id {
                            println!("Quit");
                            std::process::exit(0);
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    let event_loop = EventLoop::<UserEvent>::with_user_event().build().unwrap();

    // set a tray event handler that forwards the event and wakes up the event loop
    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::MenuEvent(event));
    }));

    let mut app = Application::new();

    // Since winit doesn't use gtk on Linux, and we need gtk for
    // the tray icon to show up, we need to spawn a thread
    // where we initialize gtk and create the tray_icon
    #[cfg(target_os = "linux")]
    std::thread::spawn(|| {
        gtk::init().unwrap();

        let _tray_icon = Application::new_tray_icon();

        gtk::main();
    });

    if let Err(err) = event_loop.run_app(&mut app) {
        println!("Error: {:?}", err);
    }
}

fn load_icon(path: &std::path::Path) -> tray_icon::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}