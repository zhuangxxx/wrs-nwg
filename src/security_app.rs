use std::{
    cell::RefCell,
    ffi::OsString,
    mem::size_of,
    thread::{self, JoinHandle},
};

use calamine::{Reader, Xlsx};
use chrono::{Local, TimeZone};
use nwd::{NwgPartial, NwgUi};
use nwg::NativeUi;
use rusqlite::Result;
use simple_excel_writer::{row, Column, Row, Workbook};

use crate::{
    db::{DbConn, Model, ModelNameType},
    security_model::SecurityModel,
};

enum SecurityFormError {
    InvalidInput(String, String),
}

#[derive(Default, NwgPartial)]
struct SecurityFormUi {
    #[nwg_layout(max_column: Some(6), max_row: Some(24))]
    form_layout: nwg::GridLayout,

    #[nwg_control(text: "水安全", h_align: nwg::HTextAlign::Center, line_height: Some(24))]
    #[nwg_layout_item(layout: form_layout, col: 0, row: 0, col_span: 6, row_span: 2)]
    title_label: nwg::RichLabel,

    #[nwg_control(text: "编号", h_align: nwg::HTextAlign::Right)]
    #[nwg_layout_item(layout: form_layout, col: 0, row: 2, col_span: 2)]
    id_label: nwg::Label,
    #[nwg_control(text: "", h_align: nwg::HTextAlign::Left)]
    #[nwg_layout_item(layout: form_layout, col: 2, row: 2, col_span: 4)]
    id_input: nwg::Label,

    #[nwg_control(text: "防洪排涝等级", h_align: nwg::HTextAlign::Right)]
    #[nwg_layout_item(layout: form_layout, col: 0, row: 3, col_span: 2)]
    level_label: nwg::Label,
    #[nwg_control(range: Some(1..5), pos: Some(1))]
    #[nwg_layout_item(layout: form_layout, col: 2, row: 3, col_span: 3)]
    level_input: nwg::TrackBar,
    #[nwg_control(text: "第一级", h_align: nwg::HTextAlign::Right)]
    #[nwg_layout_item(layout: form_layout, col: 5, row: 3)]
    level_text: nwg::Label,

    #[nwg_control(text: "河道名称", h_align: nwg::HTextAlign::Right)]
    #[nwg_layout_item(layout: form_layout, col: 0, row: 4, col_span: 2)]
    name_label: nwg::Label,
    #[nwg_control]
    #[nwg_layout_item(layout: form_layout, col: 2, row: 4, col_span: 4)]
    name_input: nwg::TextInput,

    #[nwg_control(text: "河道所属辖区", h_align: nwg::HTextAlign::Right)]
    #[nwg_layout_item(layout: form_layout, col: 0, row: 5, col_span: 2)]
    area_label: nwg::Label,
    #[nwg_control]
    #[nwg_layout_item(layout: form_layout, col: 2, row: 5, col_span: 4)]
    area_input: nwg::TextInput,

    #[nwg_control(text: "河道起点", h_align: nwg::HTextAlign::Right)]
    #[nwg_layout_item(layout: form_layout, col: 0, row: 6, col_span: 2)]
    start_label: nwg::Label,
    #[nwg_control]
    #[nwg_layout_item(layout: form_layout, col: 2, row: 6, col_span: 4)]
    start_input: nwg::TextInput,

    #[nwg_control(text: "河道终点", h_align: nwg::HTextAlign::Right)]
    #[nwg_layout_item(layout: form_layout, col: 0, row: 7, col_span: 2)]
    end_label: nwg::Label,
    #[nwg_control]
    #[nwg_layout_item(layout: form_layout, col: 2, row: 7, col_span: 4)]
    end_input: nwg::TextInput,

    #[nwg_control(text: "河道宽度(m)", h_align: nwg::HTextAlign::Right)]
    #[nwg_layout_item(layout: form_layout, col: 0, row: 8, col_span: 2)]
    river_width_label: nwg::Label,
    #[nwg_control]
    #[nwg_layout_item(layout: form_layout, col: 2, row: 8, col_span: 4)]
    river_width_input: nwg::TextInput,

    #[nwg_control(text: "边坡比", h_align: nwg::HTextAlign::Right)]
    #[nwg_layout_item(layout: form_layout, col: 0, row: 9, col_span: 2)]
    ratio_label: nwg::Label,
    #[nwg_control(text: "是", flags: "VISIBLE | GROUP")]
    #[nwg_layout_item(layout: form_layout, col: 2, row: 9)]
    ratio_radio_true: nwg::RadioButton,
    #[nwg_control(text: "否", flags: "VISIBLE")]
    #[nwg_layout_item(layout: form_layout, col: 3, row: 9)]
    ratio_radio_false: nwg::RadioButton,
    #[nwg_control(text: "0")]
    #[nwg_layout_item(layout: form_layout, col: 4, row: 9, col_span: 2)]
    ratio_input: nwg::TextInput,

    #[nwg_control(text: "设计河底高程(m)", h_align: nwg::HTextAlign::Right)]
    #[nwg_layout_item(layout: form_layout, col: 0, row: 10, col_span: 2)]
    elevation_label: nwg::Label,
    #[nwg_control]
    #[nwg_layout_item(layout: form_layout, col: 2, row: 10, col_span: 4)]
    elevation_input: nwg::TextInput,

    #[nwg_control(text: "设计洪水水位(m)", h_align: nwg::HTextAlign::Right)]
    #[nwg_layout_item(layout: form_layout, col: 0, row: 11, col_span: 2)]
    line_label: nwg::Label,
    #[nwg_control]
    #[nwg_layout_item(layout: form_layout, col: 2, row: 11, col_span: 4)]
    line_input: nwg::TextInput,

    #[nwg_control(text: "是否允许浪爬高", h_align: nwg::HTextAlign::Right)]
    #[nwg_layout_item(layout: form_layout, col: 0, row: 12, col_span: 2)]
    allow_label: nwg::Label,
    #[nwg_control(text: "是", flags: "VISIBLE | GROUP")]
    #[nwg_layout_item(layout: form_layout, col: 2, row: 12)]
    allow_radio_true: nwg::RadioButton,
    #[nwg_control(text: "否", flags: "VISIBLE")]
    #[nwg_layout_item(layout: form_layout, col: 3, row: 12)]
    allow_radio_false: nwg::RadioButton,
    #[nwg_control(text: "自定义", flags: "VISIBLE")]
    #[nwg_layout_item(layout: form_layout, col: 4, row: 12, col_span: 2)]
    allow_radio_custom: nwg::RadioButton,

    #[nwg_control(text: "安全超高(m)", h_align: nwg::HTextAlign::Right)]
    #[nwg_layout_item(layout: form_layout, col: 0, row: 13, col_span: 2)]
    safe_label: nwg::Label,
    #[nwg_control]
    #[nwg_layout_item(layout: form_layout, col: 2, row: 13, col_span: 4)]
    safe_input: nwg::TextInput,

    #[nwg_control(text: "淤积深度(m)", h_align: nwg::HTextAlign::Right)]
    #[nwg_layout_item(layout: form_layout, col: 0, row: 14, col_span: 2)]
    depth_label: nwg::Label,
    #[nwg_control]
    #[nwg_layout_item(layout: form_layout, col: 2, row: 14, col_span: 4)]
    depth_input: nwg::TextInput,

    #[nwg_control(text: "河槽宽度(m)", h_align: nwg::HTextAlign::Right)]
    #[nwg_layout_item(layout: form_layout, col: 0, row: 15, col_span: 2)]
    channel_width_label: nwg::Label,
    #[nwg_control(readonly: true)]
    #[nwg_layout_item(layout: form_layout, col: 2, row: 15, col_span: 4)]
    channel_width_input: nwg::TextInput,

    #[nwg_control(text: "淤积阈值(m)", h_align: nwg::HTextAlign::Right)]
    #[nwg_layout_item(layout: form_layout, col: 0, row: 16, col_span: 2)]
    threshold_label: nwg::Label,
    #[nwg_control(readonly: true)]
    #[nwg_layout_item(layout: form_layout, col: 2, row: 16, col_span: 4)]
    threshold_input: nwg::TextInput,

    #[nwg_control(text: "清淤判断", h_align: nwg::HTextAlign::Right)]
    #[nwg_layout_item(layout: form_layout, col: 0, row: 17, col_span: 2)]
    dredging_label: nwg::Label,
    #[nwg_control(readonly: true, flags: "VISIBLE | AUTOVSCROLL | AUTOHSCROLL")]
    #[nwg_layout_item(layout: form_layout, col: 2, row: 17, col_span: 4, row_span: 3)]
    dredging_input: nwg::TextBox,

    #[nwg_control(text: "录入时间", h_align: nwg::HTextAlign::Right)]
    #[nwg_layout_item(layout: form_layout, col: 0, row: 20, col_span: 2)]
    time_label: nwg::Label,
    #[nwg_control(text: "", h_align: nwg::HTextAlign::Left)]
    #[nwg_layout_item(layout: form_layout, col: 2, row: 20, col_span: 4)]
    time_input: nwg::Label,

    #[nwg_control(size: (120, 40), text: "计算")]
    #[nwg_layout_item(layout: form_layout, col: 4, row: 21, col_span: 2)]
    calc_button: nwg::Button,

    #[nwg_control(size: (80, 30), text: "保存")]
    #[nwg_layout_item(layout: form_layout, col: 0, row: 22)]
    save_button: nwg::Button,

    #[nwg_control(size: (80, 30), text: "重置")]
    #[nwg_layout_item(layout: form_layout, col: 2, row: 22)]
    reset_button: nwg::Button,

    #[nwg_control(size: (80, 30), text: "取消")]
    #[nwg_layout_item(layout: form_layout, col: 4, row: 22)]
    cancel_button: nwg::Button,
}

#[derive(Default, NwgUi)]
pub struct SecurityFormWindow {
    db_conn: RefCell<Option<DbConn<SecurityModel>>>,

    #[nwg_control(size: (400, 800), center: true, title: "水安全", flags: "WINDOW | VISIBLE")]
    #[nwg_events(OnWindowClose: [Self::window_close], OnInit: [Self::init_window])]
    window: nwg::Window,

    #[nwg_layout(parent: window)]
    layout: nwg::FlexboxLayout,

    #[nwg_control]
    #[nwg_layout_item(layout: layout)]
    form_frame: nwg::Frame,

    #[nwg_partial(parent: form_frame)]
    #[nwg_events(
        (level_input, OnHorizontalScroll): [Self::level_scroll],
        (ratio_radio_true, OnButtonClick): [Self::ratio_true_checked],
        (ratio_radio_false, OnButtonClick): [Self::ratio_false_checked],
        (allow_radio_true, OnButtonClick): [Self::allow_true_checked],
        (allow_radio_false, OnButtonClick): [Self::allow_false_checked],
        (allow_radio_custom, OnButtonClick): [Self::allow_custom_checked],
        (calc_button, OnButtonClick): [Self::calc_button_click],
        (save_button, OnButtonClick): [Self::save_button_click],
        (reset_button, OnButtonClick): [Self::reset_button_click],
        (cancel_button, OnButtonClick): [Self::cancel_button_click],
    )]
    security_form_ui: SecurityFormUi,
}

impl SecurityFormWindow {
    pub fn window_open(
        conn: Option<DbConn<SecurityModel>>,
        sender: nwg::NoticeSender,
    ) -> thread::JoinHandle<DbConn<SecurityModel>> {
        thread::spawn(move || {
            let app =
                Self::build_ui(Default::default()).expect("Build SecurityFormWindow UI failed.");

            *app.db_conn.borrow_mut() = conn;

            nwg::dispatch_thread_events();

            sender.notice();

            app.db_conn.take().unwrap()
        })
    }

    fn window_close(&self) {
        nwg::stop_thread_dispatch();
    }

    fn reset_model(&self) {
        self.security_form_ui.level_input.set_pos(1);
        self.security_form_ui.level_text.set_text("第一级");
        self.security_form_ui.name_input.set_text("");
        self.security_form_ui.area_input.set_text("");
        self.security_form_ui.start_input.set_text("");
        self.security_form_ui.end_input.set_text("");
        self.security_form_ui.river_width_input.set_text("");
        self.security_form_ui
            .ratio_radio_true
            .set_check_state(nwg::RadioButtonState::Unchecked);
        self.security_form_ui
            .ratio_radio_false
            .set_check_state(nwg::RadioButtonState::Unchecked);
        self.security_form_ui.ratio_input.set_text("0");
        self.security_form_ui.elevation_input.set_text("");
        self.security_form_ui.line_input.set_text("");
        self.security_form_ui
            .allow_radio_true
            .set_check_state(nwg::RadioButtonState::Unchecked);
        self.security_form_ui
            .allow_radio_false
            .set_check_state(nwg::RadioButtonState::Unchecked);
        self.security_form_ui
            .allow_radio_custom
            .set_check_state(nwg::RadioButtonState::Unchecked);
        self.security_form_ui.safe_input.set_text("");
        self.security_form_ui.depth_input.set_text("");
        self.security_form_ui.channel_width_input.set_text("");
        self.security_form_ui.threshold_input.set_text("");
        self.security_form_ui.dredging_input.set_text("");
        self.security_form_ui
            .time_input
            .set_text(format!("{}", Local::now().format("%Y-%m-%d %H:%M:%S")).as_str());
    }

    fn init_model(&self) {
        let mut conn = self.db_conn.take().unwrap();
        if let Some(model) = conn.model.take() {
            if model.id > 0 {
                self.security_form_ui
                    .id_input
                    .set_text(model.id.to_string().as_str());
            }

            self.security_form_ui
                .level_input
                .set_pos(model.level as usize);
            let level_text = match model.level {
                1 => "第一级",
                2 => "第二级",
                3 => "第三级",
                4 => "第四级",
                5 => "第五级",
                _ => "第一级",
            };
            self.security_form_ui.level_text.set_text(level_text);

            self.security_form_ui
                .name_input
                .set_text(model.name.as_str());

            self.security_form_ui
                .area_input
                .set_text(model.area.as_str());

            self.security_form_ui
                .start_input
                .set_text(model.start.as_str());

            self.security_form_ui.end_input.set_text(model.end.as_str());

            self.security_form_ui
                .river_width_input
                .set_text(model.river_width.to_string().as_str());

            if model.ratio == 0.0 {
                self.security_form_ui
                    .ratio_radio_false
                    .set_check_state(nwg::RadioButtonState::Checked);
                self.ratio_false_checked();
            } else {
                self.security_form_ui
                    .ratio_radio_true
                    .set_check_state(nwg::RadioButtonState::Checked);
                self.ratio_true_checked();
                self.security_form_ui
                    .ratio_input
                    .set_text(model.ratio.to_string().as_str());
            }

            self.security_form_ui
                .elevation_input
                .set_text(model.elevation.to_string().as_str());

            self.security_form_ui
                .line_input
                .set_text(model.line.to_string().as_str());

            match (model.level, model.safe.to_string().as_str()) {
                (1, "0.5") | (2, "0.4") | (3, "0.4") | (4, "0.3") | (5, "0.3") => {
                    self.security_form_ui
                        .allow_radio_true
                        .set_check_state(nwg::RadioButtonState::Checked);
                    self.allow_true_checked();
                }
                (1, "1.0") | (2, "0.8") | (3, "0.7") | (4, "0.6") | (5, "0.5") => {
                    self.security_form_ui
                        .allow_radio_false
                        .set_check_state(nwg::RadioButtonState::Checked);
                    self.allow_false_checked();
                }
                _ => {
                    self.security_form_ui
                        .allow_radio_custom
                        .set_check_state(nwg::RadioButtonState::Checked);
                    self.allow_custom_checked();
                }
            }

            self.security_form_ui
                .safe_input
                .set_text(model.safe.to_string().as_str());

            self.security_form_ui
                .depth_input
                .set_text(model.depth.to_string().as_str());

            self.security_form_ui
                .channel_width_input
                .set_text(model.channel_width.to_string().as_str());

            self.security_form_ui
                .threshold_input
                .set_text(model.threshold.to_string().as_str());

            self.security_form_ui
                .dredging_input
                .set_text(model.dredging.as_str());

            self.security_form_ui
                .time_input
                .set_text(format!("{}", model.time.format("%Y-%m-%d %H:%M:%S")).as_str());

            conn.set(model);
        } else {
            self.reset_model();
        }

        *self.db_conn.borrow_mut() = Some(conn);
    }

    fn init_window(&self) {
        let mut font = nwg::Font::default();
        if nwg::Font::builder()
            .size(24)
            .family("NSimSum")
            .weight(900)
            .build(&mut font)
            .is_ok()
        {
            self.security_form_ui.title_label.set_font(Some(&font));
        }

        self.security_form_ui.ratio_input.set_visible(false);
        self.security_form_ui
            .time_input
            .set_text(format!("{}", Local::now().format("%Y-%m-%d %H:%M:%S")).as_str());

        self.init_model();
    }

    fn level_scroll(&self) {
        let level_text = match self.security_form_ui.level_input.pos() {
            1 => "第一级",
            2 => "第二级",
            3 => "第三级",
            4 => "第四级",
            5 => "第五级",
            _ => "第一级",
        };
        self.security_form_ui.level_text.set_text(level_text);

        if self.security_form_ui.allow_radio_true.check_state() == nwg::RadioButtonState::Checked {
            self.allow_true_checked();
        } else if self.security_form_ui.allow_radio_false.check_state()
            == nwg::RadioButtonState::Checked
        {
            self.allow_false_checked();
        }
    }

    fn ratio_true_checked(&self) {
        if !self.security_form_ui.ratio_input.visible() {
            self.security_form_ui.ratio_input.set_text("");
        }
        self.security_form_ui.ratio_input.set_visible(true);
    }

    fn ratio_false_checked(&self) {
        self.security_form_ui.ratio_input.set_text("0");
        self.security_form_ui.ratio_input.set_visible(false);
    }

    fn allow_true_checked(&self) {
        let safe = match self.security_form_ui.level_input.pos() {
            1 => "0.5",
            2 | 3 => "0.4",
            4 | 5 => "0.3",
            _ => "0.5",
        };
        self.security_form_ui.safe_input.set_text(safe);
        self.security_form_ui.safe_input.set_readonly(true);
    }

    fn allow_false_checked(&self) {
        let safe = match self.security_form_ui.level_input.pos() {
            1 => "1",
            2 => "0.8",
            3 => "0.7",
            4 => "0.6",
            5 => "0.5",
            _ => "1",
        };
        self.security_form_ui.safe_input.set_text(safe);
        self.security_form_ui.safe_input.set_readonly(true);
    }

    fn allow_custom_checked(&self) {
        if self.security_form_ui.safe_input.readonly() {
            self.security_form_ui.safe_input.set_text("");
        }
        self.security_form_ui.safe_input.set_readonly(false);
    }

    fn check_input(&self) -> Result<(), SecurityFormError> {
        let name = self.security_form_ui.name_input.text();
        if name.is_empty() {
            return Err(SecurityFormError::InvalidInput(
                String::from("请填写河道名称"),
                String::from("name"),
            ));
        }

        let area = self.security_form_ui.area_input.text();
        if area.is_empty() {
            return Err(SecurityFormError::InvalidInput(
                String::from("请填写河道所属辖区"),
                String::from("area"),
            ));
        }
        if !area.contains("一般")
            && !area.contains("市")
            && !area.contains("县")
            && !area.contains("区")
            && !area.contains("乡")
        {
            return Err(SecurityFormError::InvalidInput(
                String::from(
                    "河道所属辖区至少要含有【市】|【县】|【区】|【乡】|【一般】关键字中的一个",
                ),
                String::from("area"),
            ));
        }

        let start = self.security_form_ui.start_input.text();
        if start.is_empty() {
            return Err(SecurityFormError::InvalidInput(
                String::from("请填写河道起点"),
                String::from("start"),
            ));
        }

        let end = self.security_form_ui.end_input.text();
        if end.is_empty() {
            return Err(SecurityFormError::InvalidInput(
                String::from("请填写河道终点"),
                String::from("end"),
            ));
        }

        let river_width = self.security_form_ui.river_width_input.text();
        if river_width.is_empty() {
            return Err(SecurityFormError::InvalidInput(
                String::from("请填写河道宽度"),
                String::from("river_width"),
            ));
        }
        if let Ok(river_width) = river_width.parse::<f32>() {
            if river_width <= 0.0 {
                return Err(SecurityFormError::InvalidInput(
                    String::from("河道宽度必须大于0"),
                    String::from("river_width"),
                ));
            }
        } else {
            return Err(SecurityFormError::InvalidInput(
                String::from("河道宽度必须为数字"),
                String::from("river_width"),
            ));
        }

        let ratio = self.security_form_ui.ratio_input.text();
        if self.security_form_ui.ratio_radio_true.check_state() == nwg::RadioButtonState::Unchecked
            && self.security_form_ui.ratio_radio_false.check_state()
                == nwg::RadioButtonState::Unchecked
        {
            return Err(SecurityFormError::InvalidInput(
                String::from("请选择是否有边坡比"),
                String::new(),
            ));
        }
        if self.security_form_ui.ratio_radio_true.check_state() == nwg::RadioButtonState::Checked {
            if ratio.is_empty() {
                self.security_form_ui.ratio_input.set_focus();
                return Err(SecurityFormError::InvalidInput(
                    String::from("请填写边坡比"),
                    String::new(),
                ));
            }
            if let Ok(ratio) = ratio.parse::<f32>() {
                if ratio == 0.0 {
                    self.security_form_ui.river_width_input.set_focus();
                    return Err(SecurityFormError::InvalidInput(
                        String::from("边坡比不能为0"),
                        String::new(),
                    ));
                }
            } else {
                self.security_form_ui.river_width_input.set_focus();
                return Err(SecurityFormError::InvalidInput(
                    String::from("边坡比必须为数字"),
                    String::new(),
                ));
            }
        }

        let elevation = self.security_form_ui.elevation_input.text();
        if elevation.is_empty() {
            return Err(SecurityFormError::InvalidInput(
                String::from("请填写河底高程"),
                String::from("elevation"),
            ));
        }
        if let Ok(elevation) = elevation.parse::<f32>() {
            if elevation <= 0.0 {
                return Err(SecurityFormError::InvalidInput(
                    String::from("河底高程必须大于0"),
                    String::from("elevation"),
                ));
            }
        } else {
            return Err(SecurityFormError::InvalidInput(
                String::from("河底高程必须为数字"),
                String::from("elevation"),
            ));
        }

        let line = self.security_form_ui.line_input.text();
        if line.is_empty() {
            return Err(SecurityFormError::InvalidInput(
                String::from("请填写洪水水位"),
                String::from("line"),
            ));
        }
        if let Ok(line) = line.parse::<f32>() {
            if line <= 0.0 {
                return Err(SecurityFormError::InvalidInput(
                    String::from("洪水水位必须大于0"),
                    String::from("line"),
                ));
            }
        } else {
            return Err(SecurityFormError::InvalidInput(
                String::from("洪水水位必须为数字"),
                String::from("line"),
            ));
        }

        if self.security_form_ui.allow_radio_true.check_state() == nwg::RadioButtonState::Unchecked
            && self.security_form_ui.allow_radio_false.check_state()
                == nwg::RadioButtonState::Unchecked
            && self.security_form_ui.allow_radio_custom.check_state()
                == nwg::RadioButtonState::Unchecked
        {
            return Err(SecurityFormError::InvalidInput(
                String::from("请选择是否允许浪爬高"),
                String::new(),
            ));
        }

        let safe = self.security_form_ui.safe_input.text();
        if safe.is_empty() {
            return Err(SecurityFormError::InvalidInput(
                String::from("请填写安全超高"),
                String::from("safe"),
            ));
        }
        if let Ok(safe) = safe.parse::<f32>() {
            if safe <= 0.0 {
                return Err(SecurityFormError::InvalidInput(
                    String::from("安全超高必须大于0"),
                    String::from("safe"),
                ));
            }
        } else {
            return Err(SecurityFormError::InvalidInput(
                String::from("安全超高必须为数字"),
                String::from("safe"),
            ));
        }

        let depth = self.security_form_ui.depth_input.text();
        if depth.is_empty() {
            return Err(SecurityFormError::InvalidInput(
                String::from("请填写淤积深度"),
                String::from("depth"),
            ));
        }
        if let Ok(depth) = depth.parse::<f32>() {
            if depth <= 0.0 {
                return Err(SecurityFormError::InvalidInput(
                    String::from("淤积深度必须大于0"),
                    String::from("depth"),
                ));
            }
        } else {
            return Err(SecurityFormError::InvalidInput(
                String::from("淤积深度必须为数字"),
                String::from("depth"),
            ));
        }
        Ok(())
    }

    fn exec_calc(&self) -> Result<(), SecurityFormError> {
        if let Err(error) = self.check_input() {
            return Err(error);
        }

        let area = self.security_form_ui.area_input.text();
        let area = if area.contains("一般") || area.contains("乡") {
            3
        } else if area.contains("县") || area.contains("区") {
            2
        } else if area.contains("市") {
            1
        } else {
            3
        };
        let ratio = if self.security_form_ui.ratio_radio_true.check_state()
            == nwg::RadioButtonState::Checked
        {
            self.security_form_ui
                .ratio_input
                .text()
                .parse::<f32>()
                .unwrap()
        } else {
            0.0
        };
        let depth = self
            .security_form_ui
            .depth_input
            .text()
            .parse::<f32>()
            .unwrap();
        let river_width = self
            .security_form_ui
            .river_width_input
            .text()
            .parse::<f32>()
            .unwrap();

        self.security_form_ui
            .channel_width_input
            .set_text(river_width.to_string().as_str());

        if ratio == 0.0 {
            if area == 3 {
                if depth <= 47.0 {
                    self.security_form_ui.threshold_input.set_text("47");
                    self.security_form_ui.dredging_input.set_text("不需要清淤.");
                } else if depth <= 61.0 {
                    self.security_form_ui.threshold_input.set_text("61");
                    self.security_form_ui
                        .dredging_input
                        .set_text("建议对该河道进行清淤.");
                } else {
                    self.security_form_ui.threshold_input.set_text("61");
                    self.security_form_ui
                        .dredging_input
                        .set_text("需要对该河道进行清淤.");
                }
            } else if area == 2 {
                self.security_form_ui.threshold_input.set_text("19");
                if depth <= 33.0 {
                    self.security_form_ui.dredging_input.set_text("不需要清淤.");
                } else {
                    self.security_form_ui
                        .dredging_input
                        .set_text("需要对该河道进行清淤.");
                }
            } else if area == 1 {
                self.security_form_ui.threshold_input.set_text("19");
                if depth <= 19.0 {
                    self.security_form_ui.dredging_input.set_text("不需要清淤.");
                } else {
                    self.security_form_ui
                        .dredging_input
                        .set_text("需要对该河道进行清淤.");
                }
            } else {
                if depth <= 47.0 {
                    self.security_form_ui.threshold_input.set_text("47");
                    self.security_form_ui.dredging_input.set_text("不需要清淤.");
                } else if depth <= 61.0 {
                    self.security_form_ui.threshold_input.set_text("61");
                    self.security_form_ui
                        .dredging_input
                        .set_text("建议对该河道进行清淤.");
                } else {
                    self.security_form_ui.threshold_input.set_text("61");
                    self.security_form_ui
                        .dredging_input
                        .set_text("需要对该河道进行清淤.");
                }
            }
        } else {
            let line = self
                .security_form_ui
                .line_input
                .text()
                .parse::<f32>()
                .unwrap();
            let safe = self
                .security_form_ui
                .safe_input
                .text()
                .parse::<f32>()
                .unwrap();

            let slope_ratio = 1.0 / ratio;

            let channel_width = river_width - 2.0 * (line + safe) * ratio;
            self.security_form_ui
                .channel_width_input
                .set_text(channel_width.to_string().as_str());

            let threshold = ((((0.04 / ratio)
                * ((line + safe).powi(2) * slope_ratio + channel_width * (line + safe))
                + channel_width.powi(2) / 4.0 * slope_ratio.powi(2))
                - channel_width / 2.0 * slope_ratio)
                * 100.0)
                .round()
                / 100.0;
            if area == 3 || area == 2 || area == 1 {
                if depth <= threshold {
                    self.security_form_ui
                        .threshold_input
                        .set_text(threshold.to_string().as_str());
                    self.security_form_ui.dredging_input.set_text("不需要清淤.");
                } else {
                    self.security_form_ui
                        .threshold_input
                        .set_text(threshold.to_string().as_str());
                    self.security_form_ui
                        .dredging_input
                        .set_text("需要对该河道进行清淤.");
                }
            } else {
                self.security_form_ui.threshold_input.set_text("0");
                self.security_form_ui
                    .dredging_input
                    .set_text("计算错误，请检查输入数据.");
            }
        }
        Ok(())
    }

    fn calc_button_click(&self) {
        if let Err(error) = self.exec_calc() {
            match error {
                SecurityFormError::InvalidInput(message, input) => {
                    nwg::simple_message("无效输入", message.as_str());
                    match input.as_str() {
                        "name" => self.security_form_ui.name_input.set_focus(),
                        "area" => self.security_form_ui.area_input.set_focus(),
                        "start" => self.security_form_ui.start_input.set_focus(),
                        "end" => self.security_form_ui.end_input.set_focus(),
                        "river_width" => self.security_form_ui.river_width_input.set_focus(),
                        "ratio" => self.security_form_ui.ratio_input.set_focus(),
                        "elevation" => self.security_form_ui.elevation_input.set_focus(),
                        "line" => self.security_form_ui.line_input.set_focus(),
                        "safe" => self.security_form_ui.safe_input.set_focus(),
                        "depth" => self.security_form_ui.depth_input.set_focus(),
                        _ => {}
                    }
                }
            }
        }
    }

    fn save_button_click(&self) {
        if nwg::modal_message(
            &self.window,
            &nwg::MessageParams {
                title: "确认",
                content: "确定保存？",
                buttons: nwg::MessageButtons::OkCancel,
                icons: nwg::MessageIcons::Question,
            },
        ) == nwg::MessageChoice::Ok
        {
            if let Err(error) = self.exec_calc() {
                match error {
                    SecurityFormError::InvalidInput(message, input) => {
                        nwg::simple_message("无效输入", message.as_str());
                        match input.as_str() {
                            "name" => self.security_form_ui.name_input.set_focus(),
                            "area" => self.security_form_ui.area_input.set_focus(),
                            "start" => self.security_form_ui.start_input.set_focus(),
                            "end" => self.security_form_ui.end_input.set_focus(),
                            "river_width" => self.security_form_ui.river_width_input.set_focus(),
                            "ratio" => self.security_form_ui.ratio_input.set_focus(),
                            "elevation" => self.security_form_ui.elevation_input.set_focus(),
                            "line" => self.security_form_ui.line_input.set_focus(),
                            "safe" => self.security_form_ui.safe_input.set_focus(),
                            "depth" => self.security_form_ui.depth_input.set_focus(),
                            _ => {}
                        }
                    }
                }
            } else {
                let mut conn = self.db_conn.take().unwrap();
                let mut model = match conn.model.take() {
                    Some(model) => model,
                    None => SecurityModel::default(),
                };

                if !self.security_form_ui.id_input.text().is_empty() {
                    model.id = self.security_form_ui.id_input.text().parse().unwrap();
                }
                model.level = self.security_form_ui.level_input.pos() as u32;
                model.name = self.security_form_ui.name_input.text();
                model.area = self.security_form_ui.area_input.text();
                model.start = self.security_form_ui.start_input.text();
                model.end = self.security_form_ui.end_input.text();
                model.river_width = self
                    .security_form_ui
                    .river_width_input
                    .text()
                    .parse()
                    .unwrap();
                model.ratio = self.security_form_ui.ratio_input.text().parse().unwrap();
                model.elevation = self
                    .security_form_ui
                    .elevation_input
                    .text()
                    .parse()
                    .unwrap();
                model.line = self.security_form_ui.line_input.text().parse().unwrap();
                model.allow = self.security_form_ui.safe_input.text().parse().unwrap();
                model.safe = self.security_form_ui.safe_input.text().parse().unwrap();
                model.depth = self.security_form_ui.depth_input.text().parse().unwrap();
                model.channel_width = self
                    .security_form_ui
                    .channel_width_input
                    .text()
                    .parse()
                    .unwrap();
                model.threshold = self
                    .security_form_ui
                    .threshold_input
                    .text()
                    .parse()
                    .unwrap();
                model.dredging = self.security_form_ui.dredging_input.text().parse().unwrap();
                model.time = Local::now();

                let id = model.id;
                conn.set(model);
                match if id > 0 { conn.update() } else { conn.insert() } {
                    Ok(num) => nwg::simple_message(
                        "提示",
                        if num == 1 {
                            "保存成功"
                        } else {
                            "保存失败"
                        },
                    ),
                    Err(error) => nwg::simple_message("错误", error.to_string().as_str()),
                };

                *self.db_conn.borrow_mut() = Some(conn);

                self.window.close();
            }
        }
    }

    fn reset_button_click(&self) {
        self.reset_model();
    }

    fn cancel_button_click(&self) {
        self.window.close();
    }
}

#[derive(Default, NwgUi)]
pub struct SecurityApp {
    db_conn: RefCell<Option<DbConn<SecurityModel>>>,
    security_window_handle: RefCell<Option<JoinHandle<DbConn<SecurityModel>>>>,

    #[nwg_control(size: (900, 600), center: true, title: "水安全", flags: "MAIN_WINDOW | VISIBLE")]
    #[nwg_events(OnWindowClose: [Self::window_close], OnInit: [Self::init_data_view])]
    window: nwg::Window,

    #[nwg_control]
    #[nwg_events(OnNotice: [Self::security_form_notice])]
    security_form_notice: nwg::Notice,

    #[nwg_control(text: "导入")]
    #[nwg_events(OnMenuOpen: [Self::import_menu_open])]
    import_menu: nwg::Menu,

    #[nwg_control(text: "导出")]
    #[nwg_events(OnMenuOpen: [Self::export_menu_open])]
    export_menu: nwg::Menu,

    #[nwg_control(text: "新增")]
    #[nwg_events(OnMenuOpen: [Self::create_menu_open])]
    create_menu: nwg::Menu,

    #[nwg_control(popup: true)]
    right_click_menu: nwg::Menu,

    #[nwg_control(text: "刷新", parent: right_click_menu)]
    #[nwg_events(OnMenuItemSelected: [Self::reload_menu_selected])]
    reload_menu: nwg::MenuItem,

    #[nwg_control(text: "修改", parent: right_click_menu)]
    #[nwg_events(OnMenuItemSelected: [Self::update_menu_selected])]
    update_menu: nwg::MenuItem,

    #[nwg_control(text: "删除", parent: right_click_menu)]
    #[nwg_events(OnMenuItemSelected: [Self::delete_menu_selected])]
    delete_menu: nwg::MenuItem,

    #[nwg_layout(parent: window)]
    layout: nwg::GridLayout,

    #[nwg_control(size: (850, 550), list_style: nwg::ListViewStyle::Detailed, focus: true,
        ex_flags: nwg::ListViewExFlags::GRID | nwg::ListViewExFlags::FULL_ROW_SELECT,
    )]
    #[nwg_layout_item(layout: layout, col: 0, col_span: 1, row: 0, row_span: 1)]
    #[nwg_events(OnListViewRightClick: [Self::right_click_menu_popup], OnListViewDoubleClick: [Self::update_menu_selected])]
    data_view: nwg::ListView,
}

impl SecurityApp {
    pub fn window_open() -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let app = Self::build_ui(Default::default()).expect("Build SecurityApp UI failed.");

            *app.db_conn.borrow_mut() = Some(DbConn::new());

            nwg::dispatch_thread_events();
        })
    }

    fn window_close(&self) {
        nwg::stop_thread_dispatch();
    }

    fn init_data_view(&self) {
        let data_view = &self.data_view;

        for header in SecurityModel::get_names(ModelNameType::Header) {
            data_view.insert_column(header);
        }

        data_view.set_headers_enabled(true);

        self.load_data_view();
    }

    fn load_data_view(&self) {
        let mut conn = self.db_conn.take().unwrap();
        if let Ok(models) = conn.select() {
            for model in models {
                let data_view = &self.data_view;
                data_view.insert_items_row(
                    None,
                    &[
                        model.id.to_string(),
                        String::from(match model.level {
                            1 => "第一级",
                            2 => "第二级",
                            3 => "第三级",
                            4 => "第四级",
                            5 => "第五级",
                            _ => "第一级",
                        }),
                        model.name,
                        model.area,
                        model.start,
                        model.end,
                        model.river_width.to_string(),
                        model.elevation.to_string(),
                        model.ratio.to_string(),
                        model.line.to_string(),
                        String::from(if model.allow == 0.8 {
                            "是"
                        } else if model.allow == 0.4 {
                            "否"
                        } else {
                            "自定义"
                        }),
                        model.safe.to_string(),
                        model.depth.to_string(),
                        model.channel_width.to_string(),
                        model.threshold.to_string(),
                        model.dredging,
                        format!("{}", model.time.format("%Y-%m-%d %H:%M:%S")),
                    ],
                );
            }
        }

        *self.db_conn.borrow_mut() = Some(conn);
    }

    fn security_form_notice(&self) {
        let handle = self.security_window_handle.take();
        if let Some(handle) = handle {
            if let Ok(mut conn) = handle.join() {
                *conn.model = None;

                *self.db_conn.borrow_mut() = Some(conn);
                *self.security_window_handle.borrow_mut() = None;

                self.reload_menu_selected();
            }
        }
    }

    // TODO 分文件类型导入
    // TODO 自动识别工作表
    fn import_menu_open(&self) {
        let mut import_file_dialog = nwg::FileDialog::default();

        if let Ok(_) = nwg::FileDialog::builder()
            .title("请选择导入文件")
            .action(nwg::FileDialogAction::Open)
            .filters("Excel文件(*.xls;*.xlsx;*.xlsm;*.xlsb;*.xla;*.xlam)")
            .build(&mut import_file_dialog)
        {
            let mut models = vec![];
            if import_file_dialog.run(Some(&self.window)) {
                if let Ok(import_file) = import_file_dialog.get_selected_item() {
                    if let Ok(mut workbook) =
                        calamine::open_workbook::<Xlsx<_>, OsString>(import_file)
                    {
                        if let Some(Ok(range)) = workbook.worksheet_range("Sheet1") {
                            let mut current_row = 0usize;
                            let mut columns = vec![];
                            for row in range.rows() {
                                if current_row == 0 {
                                    for cell in row {
                                        let column = match cell.get_string() {
                                            Some("编号") => "id",
                                            Some("河道防洪排涝等级") => "level",
                                            Some("河道名称") => "name",
                                            Some("河道所属辖区") => "area",
                                            Some("河道起点") => "start",
                                            Some("河道终点") => "end",
                                            Some("河道宽度(m)") => "river_width",
                                            Some("边坡比") => "ratio",
                                            Some("设计河底高程(m)") => "elevation",
                                            Some("设计洪水水位(m)") => "line",
                                            Some("是否允许浪爬高") => "allow",
                                            Some("安全超高(m)") => "safe",
                                            Some("淤积深度(m)") => "depth",
                                            Some("河槽宽度(m)") => "channel_width",
                                            Some("淤积阈值(m)") => "threshold",
                                            Some("清淤判断") => "dredging",
                                            Some("录入时间") => "time",
                                            _ => "",
                                        };
                                        columns.push(column);
                                    }
                                } else {
                                    let mut current_cell = 0usize;
                                    let mut model = SecurityModel::default();
                                    for cell in row {
                                        if current_cell >= columns.len() {
                                            continue;
                                        }
                                        match columns[current_cell] {
                                            "id" => {
                                                if let Some(id) = cell.get_int() {
                                                    model.id = id as u32;
                                                }
                                            }
                                            "level" => {
                                                if let Some(level) = cell.get_int() {
                                                    model.level = level as u32;
                                                }
                                            }
                                            "name" => {
                                                if let Some(name) = cell.get_string() {
                                                    model.name = String::from(name);
                                                }
                                            }
                                            "area" => {
                                                if let Some(area) = cell.get_string() {
                                                    model.area = String::from(area);
                                                }
                                            }
                                            "start" => {
                                                if let Some(start) = cell.get_string() {
                                                    model.start = String::from(start);
                                                }
                                            }
                                            "end" => {
                                                if let Some(end) = cell.get_string() {
                                                    model.end = String::from(end);
                                                }
                                            }
                                            "river_width" => {
                                                if let Some(river_width) = cell.get_float() {
                                                    model.river_width = river_width as f32;
                                                }
                                            }
                                            "ratio" => {
                                                if let Some(ratio) = cell.get_float() {
                                                    model.ratio = ratio as f32;
                                                }
                                            }
                                            "elevation" => {
                                                if let Some(elevation) = cell.get_float() {
                                                    model.elevation = elevation as f32;
                                                }
                                            }
                                            "line" => {
                                                if let Some(line) = cell.get_float() {
                                                    model.line = line as f32;
                                                }
                                            }
                                            "allow" => {
                                                if let Some(allow) = cell.get_float() {
                                                    model.allow = allow as f32;
                                                }
                                            }
                                            "safe" => {
                                                if let Some(safe) = cell.get_float() {
                                                    model.safe = safe as f32;
                                                }
                                            }
                                            "depth" => {
                                                if let Some(depth) = cell.get_float() {
                                                    model.depth = depth as f32;
                                                }
                                            }
                                            "channel_width" => {
                                                if let Some(channel_width) = cell.get_float() {
                                                    model.channel_width = channel_width as f32;
                                                }
                                            }
                                            "threshold" => {
                                                if let Some(threshold) = cell.get_float() {
                                                    model.threshold = threshold as f32;
                                                }
                                            }
                                            "dredging" => {
                                                if let Some(dredging) = cell.get_string() {
                                                    model.dredging = String::from(dredging);
                                                }
                                            }
                                            "time" => {
                                                if let Some(time) = cell.get_string() {
                                                    model.time = match Local.datetime_from_str(
                                                        time,
                                                        "%Y-%m-%d %H:%M:%S",
                                                    ) {
                                                        Ok(time) => time,
                                                        _ => Local::now(),
                                                    };
                                                }
                                            }
                                            _ => {}
                                        }
                                        current_cell += 1;
                                    }
                                    models.push(model);
                                }
                                current_row += 1;
                            }
                        }
                    }
                    let mut conn = self.db_conn.take().unwrap();
                    let (mut insert_num, mut update_num, mut failed_num) = (0u32, 0u32, 0u32);
                    for mut model in models {
                        if let Ok(id) = conn.instance.query_row(
                            "SELECT id FROM water_security WHERE name=?1 and area=?2",
                            [model.name.as_str(), model.area.as_str()],
                            |row| row.get(0),
                        ) {
                            model.id = id;
                        } else {
                            if model.id > 0 {
                                match conn.instance.query_row(
                                    "SELECT id FROM water_security WHERE id=?1",
                                    [model.id],
                                    |row| -> Result<u32> { row.get(0) },
                                ) {
                                    Err(_) => model.id = 0,
                                    _ => {}
                                }
                            }
                        }
                        let id = model.id;
                        conn.set(model);
                        if let Ok(num) = if id > 0 { conn.update() } else { conn.insert() } {
                            if num == 1 {
                                if id > 0 {
                                    update_num += 1;
                                } else {
                                    insert_num += 1;
                                }
                            } else {
                                failed_num += 1;
                            }
                        } else {
                            failed_num += 1;
                        }
                    }

                    *self.db_conn.borrow_mut() = Some(conn);

                    nwg::simple_message(
                        "导入完成",
                        format!(
                            "导入完成，新增{}条，更新{}条，失败{}条",
                            insert_num, update_num, failed_num
                        )
                        .as_str(),
                    );

                    self.reload_menu_selected();
                }
            }
        }
    }

    fn export_menu_open(&self) {
        let mut export_file_dialog = nwg::FileDialog::default();

        if let Ok(_) = nwg::FileDialog::builder()
            .title("请选择导出位置")
            .action(nwg::FileDialogAction::Save)
            .filters("Excel文件(*.xls;*.xlsx;*.xlsm;*.xlsb;*.xla;*.xlam)")
            .build(&mut export_file_dialog)
        {
            if export_file_dialog.run(Some(&self.window)) {
                if let Ok(export_file) = export_file_dialog.get_selected_item() {
                    let mut workbook = Workbook::create(export_file.to_str().unwrap());
                    let mut sheet = workbook.create_sheet("Sheet1");

                    sheet.add_column(Column { width: 30.0 });
                    sheet.add_column(Column { width: 30.0 });
                    sheet.add_column(Column { width: 30.0 });
                    sheet.add_column(Column { width: 30.0 });
                    sheet.add_column(Column { width: 30.0 });
                    sheet.add_column(Column { width: 30.0 });
                    sheet.add_column(Column { width: 30.0 });
                    sheet.add_column(Column { width: 30.0 });
                    sheet.add_column(Column { width: 30.0 });
                    sheet.add_column(Column { width: 30.0 });
                    sheet.add_column(Column { width: 30.0 });
                    sheet.add_column(Column { width: 30.0 });
                    sheet.add_column(Column { width: 30.0 });
                    sheet.add_column(Column { width: 30.0 });
                    sheet.add_column(Column { width: 30.0 });
                    sheet.add_column(Column { width: 30.0 });
                    sheet.add_column(Column { width: 30.0 });

                    if let Ok(_) = workbook.write_sheet(&mut sheet, |sheet_writer| {
                        sheet_writer.append_row(row![
                            "编号",
                            "河道防洪排涝等级",
                            "河道名称",
                            "河道所属辖区",
                            "河道起点",
                            "河道终点",
                            "河道宽度(m)",
                            "边坡比",
                            "设计河底高程(m)",
                            "设计洪水水位(m)",
                            "是否允许浪爬高",
                            "安全超高(m)",
                            "淤积深度(m)",
                            "河槽宽度(m)",
                            "淤积阈值(m)",
                            "清淤判断",
                            "录入时间"
                        ])?;
                        let mut conn = self.db_conn.take().unwrap();
                        if let Ok(security_models) = conn.select() {
                            let mut row_num = 0usize;
                            for model in security_models {
                                sheet_writer.append_row(row![
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
                                    format!("{}", model.time.format("%Y-%m-%d %H:%M:%S"))
                                ])?;
                                row_num += 1;
                            }
                            nwg::simple_message(
                                "导出",
                                format!("导出完成，共{}条数据", row_num).as_str(),
                            );
                        }

                        *self.db_conn.borrow_mut() = Some(conn);

                        Ok(())
                    }) {}

                    if let Ok(_) = workbook.close() {}
                }
            }
        }
    }

    fn create_menu_open(&self) {
        let conn = self.db_conn.take();
        *self.security_window_handle.borrow_mut() = Some(SecurityFormWindow::window_open(
            conn,
            self.security_form_notice.sender(),
        ));
    }

    fn right_click_menu_popup(&self) {
        let (x, y) = nwg::GlobalCursor::position();
        self.right_click_menu.popup(x, y);
    }

    fn reload_menu_selected(&self) {
        self.data_view.clear();
        self.load_data_view();
    }

    fn update_menu_selected(&self) {
        if let Some(index) = self.data_view.selected_item() {
            if let Some(item) = self.data_view.item(index, 0, size_of::<u32>()) {
                let mut conn = self.db_conn.take().unwrap();
                if let Ok(model) = conn.find_by_id(item.text.parse().unwrap()) {
                    conn.set(model);
                }
                *self.security_window_handle.borrow_mut() = Some(SecurityFormWindow::window_open(
                    Some(conn),
                    self.security_form_notice.sender(),
                ));
            }
        }
    }

    fn delete_menu_selected(&self) {
        if let Some(index) = self.data_view.selected_item() {
            if let Some(item) = self.data_view.item(index, 0, size_of::<u32>()) {
                if !item.text.is_empty() {
                    if nwg::modal_message(
                        &self.window,
                        &nwg::MessageParams {
                            title: "确认",
                            content: "删除后将无法恢复，确定删除？",
                            buttons: nwg::MessageButtons::OkCancel,
                            icons: nwg::MessageIcons::Question,
                        },
                    ) == nwg::MessageChoice::Ok
                    {
                        let mut conn = self.db_conn.take().unwrap();
                        if let Ok(model) = conn.find_by_id(item.text.parse().unwrap()) {
                            conn.set(model);
                            match conn.delete() {
                                Ok(num) => nwg::simple_message(
                                    "提示",
                                    if num == 1 {
                                        "删除成功"
                                    } else {
                                        "删除失败"
                                    },
                                ),
                                Err(error) => {
                                    nwg::simple_message("错误", error.to_string().as_str())
                                }
                            };
                        }

                        *self.db_conn.borrow_mut() = Some(conn);

                        self.reload_menu_selected();
                    }
                }
            }
        }
    }
}
