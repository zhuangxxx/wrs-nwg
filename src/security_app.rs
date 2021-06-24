use std::thread;

use nwd::NwgUi;
use nwg::NativeUi;

#[derive(Default, NwgUi)]
pub struct SecurityApp {
    #[nwg_control(size:(900,600), center:true, title:"水安全", flags:"MAIN_WINDOW|VISIBLE")]
    #[nwg_events(OnWindowClose:[Self::window_close])]
    window: nwg::Window,

    #[nwg_control(text: "导入")]
    #[nwg_events(OnMenuOpen:[Self::import_menu_open])]
    import_menu: nwg::Menu,

    #[nwg_control(text: "导出")]
    #[nwg_events(OnMenuOpen:[Self::export_menu_open])]
    export_menu: nwg::Menu,
}

impl SecurityApp {
    pub fn window_open() -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let _app = Self::build_ui(Default::default()).expect("Failed to build SecurityApp UI");
            nwg::dispatch_thread_events();
        })
    }

    fn window_close(&self) {
        nwg::stop_thread_dispatch();
    }

    fn import_menu_open(&self) {
        nwg::simple_message("导入", "TODO:数据导入功能");
    }

    fn export_menu_open(&self) {
        nwg::simple_message("导出", "TODO:数据导出功能");
    }
}
