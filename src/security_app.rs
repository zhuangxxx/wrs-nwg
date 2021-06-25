use std::thread;

use chrono::{DateTime, Utc};
use nwd::NwgUi;
use nwg::NativeUi;
use rusqlite::{Connection, Result};

struct SecurityModel {
    id: u32,
    level: u32,
    name: String,
    area: String,
    start: String,
    end: String,
    river_width: f32,
    elevation: f32,
    ratio: f32,
    line: f32,
    allow: f32,
    safe: f32,
    depth: f32,
    channel_width: f32,
    threshold: f32,
    dredging: String,
    time: DateTime<Utc>,
}

#[derive(Default, NwgUi)]
pub struct SecurityApp {
    #[nwg_control(size: (900, 600), center: true, title: "水安全", flags: "MAIN_WINDOW | VISIBLE")]
    #[nwg_events(OnWindowClose: [Self::window_close], OnInit: [Self::init_data_view])]
    window: nwg::Window,

    #[nwg_control(text: "导入")]
    #[nwg_events(OnMenuOpen: [Self::import_menu_open])]
    import_menu: nwg::Menu,

    #[nwg_control(text: "导出")]
    #[nwg_events(OnMenuOpen: [Self::export_menu_open])]
    export_menu: nwg::Menu,

    #[nwg_layout(parent: window)]
    layout: nwg::GridLayout,

    #[nwg_control(size: (850, 550), list_style: nwg::ListViewStyle::Detailed, focus: true,
        ex_flags: nwg::ListViewExFlags::GRID | nwg::ListViewExFlags::FULL_ROW_SELECT,
    )]
    #[nwg_layout_item(layout: layout, col: 0, col_span: 1, row: 0, row_span: 1)]
    data_view: nwg::ListView,
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

    fn init_data_view(&self) {
        let data_view = &self.data_view;

        data_view.insert_column("编号");
        data_view.insert_column("河道防洪排涝等级");
        data_view.insert_column("河道名称");
        data_view.insert_column("河道所属辖区");
        data_view.insert_column("河道起点");
        data_view.insert_column("河道终点");
        data_view.insert_column("河道宽度(m)");
        data_view.insert_column("边坡比");
        data_view.insert_column("设计河底高程(m)");
        data_view.insert_column("设计洪水水位(m)");
        data_view.insert_column("是否允许浪爬高");
        data_view.insert_column("安全超高(m)");
        data_view.insert_column("淤积深度(m)");
        data_view.insert_column("河槽宽度(m)");
        data_view.insert_column("淤积阈值(m)");
        data_view.insert_column("清淤判断");
        data_view.insert_column("录入时间");

        data_view.set_headers_enabled(true);

        self.load_data_view();
    }

    fn load_data_view(&self) {
        if let Ok(conn) = Connection::open("./water-resources.db") {
            if let Ok(mut stmt) = conn.prepare("SELECT * FROM water_security") {
                if let Ok(mut security_models) =
                    stmt.query_and_then([], |row| -> Result<SecurityModel> {
                        Ok(SecurityModel {
                            id: row.get(0)?,
                            level: row.get(1)?,
                            name: row.get(2)?,
                            area: row.get(3)?,
                            start: row.get(4)?,
                            end: row.get(5)?,
                            river_width: row.get(6)?,
                            elevation: row.get(7)?,
                            ratio: row.get(8)?,
                            line: row.get(9)?,
                            allow: row.get(10)?,
                            safe: row.get(11)?,
                            depth: row.get(12)?,
                            channel_width: row.get(13)?,
                            threshold: row.get(14)?,
                            dredging: row.get(15)?,
                            time: row.get(16)?,
                        })
                    })
                {
                    while let Some(Ok(model)) = security_models.next() {
                        let data_view = &self.data_view;

                        data_view.insert_items_row(
                            None,
                            &[
                                model.id.to_string(),
                                model.level.to_string(),
                                model.name,
                                model.area,
                                model.start,
                                model.end,
                                model.river_width.to_string(),
                                model.elevation.to_string(),
                                model.ratio.to_string(),
                                model.line.to_string(),
                                model.allow.to_string(),
                                model.safe.to_string(),
                                model.depth.to_string(),
                                model.channel_width.to_string(),
                                model.threshold.to_string(),
                                model.dredging,
                                model.time.to_rfc3339(),
                            ],
                        );
                    }
                }
            }
        }
    }

    fn import_menu_open(&self) {
        nwg::simple_message("导入", "TODO:数据导入功能");
    }

    fn export_menu_open(&self) {
        nwg::simple_message("导出", "TODO:数据导出功能");
    }
}
