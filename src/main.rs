#![windows_subsystem = "windows"]

use nwd::NwgUi;
use nwg::NativeUi;

#[derive(Default, NwgUi)]
pub struct BasicApp {
    #[nwg_control(size:(900,600), center:true, title:"水资源", flags:"MAIN_WINDOW|VISIBLE")]
    #[nwg_events(OnWindowClose:[Self::window_close])]
    window: nwg::Window,

    #[nwg_control(text: "导入")]
    #[nwg_events(OnMenuOpen:[Self::import_menu_open])]
    import_menu: nwg::Menu,

    #[nwg_control(text: "导出")]
    #[nwg_events(OnMenuOpen:[Self::export_menu_open])]
    export_menu: nwg::Menu,

    #[nwg_control(size:(80,30), position:(20, 20), text:"水安全")]
    #[nwg_events(OnButtonClick:[Self::security_button_click])]
    security_button: nwg::Button,

    #[nwg_control(size:(80,30), position:(120, 20), text:"水环境")]
    #[nwg_events(OnButtonClick:[Self::environment_button_click])]
    environment_button: nwg::Button,

    #[nwg_control(size:(80,30), position:(220, 20), text:"退出")]
    #[nwg_events(OnButtonClick:[Self::quit_button_click])]
    quit_button: nwg::Button,
}

impl BasicApp {
    fn window_close(&self) {
        nwg::stop_thread_dispatch();
    }

    fn import_menu_open(&self) {
        nwg::simple_message("导入", "TODO:数据导入功能");
    }

    fn export_menu_open(&self) {
        nwg::simple_message("导出", "TODO:数据导出功能");
    }

    fn security_button_click(&self) {
        nwg::simple_message("水安全", "TODO:水安全窗口");
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
    nwg::Font::set_global_family("Microsoft YaHei UI").expect("Failed to set default font");

    let _app = BasicApp::build_ui(Default::default()).expect("Failed to build UI");

    nwg::dispatch_thread_events();
}
