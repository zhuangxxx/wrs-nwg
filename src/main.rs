#![windows_subsystem = "windows"]

use std::{cell::RefCell, thread};

use nwd::NwgUi;
use nwg::NativeUi;

mod security_app;

#[derive(Default, NwgUi)]
pub struct BasicApp {
    security_handle: RefCell<Option<thread::JoinHandle<()>>>,

    #[nwg_control(size: (900, 600), center: true, title: "水资源", flags: "MAIN_WINDOW | VISIBLE")]
    #[nwg_events(OnWindowClose: [Self::window_close], OnResize: [Self::window_resize], OnWindowMaximize: [Self::window_resize])]
    window: nwg::Window,

    #[nwg_control(size: (120, 40), position: (900 / 2 - 60 - 160, 600 / 10 * 8), text: "水安全")]
    #[nwg_events(OnButtonClick: [Self::security_button_click])]
    security_button: nwg::Button,

    #[nwg_control(size: (120, 40), position: (900 / 2 - 60, 600 / 10 * 8), text: "水环境")]
    #[nwg_events(OnButtonClick: [Self::environment_button_click])]
    environment_button: nwg::Button,

    #[nwg_control(size: (120, 40), position: (900 / 2 - 60 + 160, 600 / 10 * 8), text: "退出")]
    #[nwg_events(OnButtonClick: [Self::quit_button_click])]
    quit_button: nwg::Button,
}

impl BasicApp {
    fn window_close(&self) {
        nwg::stop_thread_dispatch();
    }

    fn window_resize(&self) {
        let (width, height) = self.window.size();
        let width = width as i32;
        let height = height as i32;
        self.security_button
            .set_position(width / 2 - 60 - 160, height / 10 * 8);
        self.environment_button
            .set_position(width / 2 - 60, height / 10 * 8);
        self.quit_button
            .set_position(width / 2 - 60 + 160, height / 10 * 8);
    }

    fn security_button_click(&self) {
        *self.security_handle.borrow_mut() = Some(security_app::SecurityApp::window_open());
        self.window.set_visible(false);

        let handle = self.security_handle.borrow_mut().take();
        if let Some(handle) = handle {
            handle.join().unwrap();
            self.window.set_visible(true);
        }
    }

    fn environment_button_click(&self) {
        nwg::simple_message("水环境", "TODO:水环境窗口");
    }

    fn quit_button_click(&self) {
        self.window.close();
    }
}

/// # Compile on Windows
/// ```
/// cargo rustc -- -Clink-args="/SUBSYSTEM:WINDOWS /ENTRY:mainCRTStartup"
/// ```
/// set entry point named WinMain
fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");

    // nwg::Font::set_global_family("Microsoft YaHei UI").expect("Failed to set default font");

    let mut font = nwg::Font::default();
    if nwg::Font::builder()
        .size(16)
        .family("NSimSum")
        .weight(400)
        .build(&mut font)
        .is_ok()
    {
        nwg::Font::set_global_default(Some(font));
    }

    let _app = BasicApp::build_ui(Default::default()).expect("Failed to build UI");

    nwg::dispatch_thread_events();
}
