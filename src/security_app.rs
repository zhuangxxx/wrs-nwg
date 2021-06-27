use std::{
    cell::RefCell,
    thread::{self, JoinHandle},
};

use chrono::{DateTime, SecondsFormat, Utc};
use nwd::{NwgPartial, NwgUi};
use nwg::NativeUi;
use rusqlite::{params, Connection};
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct SecurityModel {
    id: u32,
    level: u32,
    name: String,
    area: String,
    start: String,
    end: String,
    river_width: f32,
    ratio: f32,
    elevation: f32,
    line: f32,
    allow: f32,
    safe: f32,
    depth: f32,
    channel_width: f32,
    threshold: f32,
    dredging: String,
    time: DateTime<Utc>,
}

impl Default for SecurityModel {
    fn default() -> Self {
        Self {
            id: Default::default(),
            level: Default::default(),
            name: Default::default(),
            area: Default::default(),
            start: Default::default(),
            end: Default::default(),
            river_width: Default::default(),
            ratio: Default::default(),
            elevation: Default::default(),
            line: Default::default(),
            allow: Default::default(),
            safe: Default::default(),
            depth: Default::default(),
            channel_width: Default::default(),
            threshold: Default::default(),
            dredging: Default::default(),
            time: Utc::now(),
        }
    }
}

enum SecurityFormError {
    InvalidInput(String, String),
}

// TODO 合并如SecurityFormWindow
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
    model: RefCell<Option<SecurityModel>>,

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
    pub fn window_open(model: Option<SecurityModel>) -> thread::JoinHandle<SecurityModel> {
        thread::spawn(move || {
            let app = Self::build_ui(Default::default()).expect("Failed to build SecurityApp UI");

            *app.model.borrow_mut() = model;

            nwg::dispatch_thread_events();

            app.model.take().unwrap_or(Default::default())
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
        self.security_form_ui.time_input.set_text(
            Utc::now()
                .to_rfc3339_opts(SecondsFormat::Secs, true)
                .as_str(),
        );
    }

    fn init_model(&self) {
        let model = self.model.take();
        if let Some(model) = model {
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

            self.security_form_ui.time_input.set_text(
                model
                    .time
                    .to_rfc3339_opts(SecondsFormat::Secs, true)
                    .as_str(),
            );

            *self.model.borrow_mut() = Some(model);
        } else {
            self.reset_model();
            *self.model.borrow_mut() = None;
        }
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
        self.security_form_ui.time_input.set_text(
            Utc::now()
                .to_rfc3339_opts(SecondsFormat::Secs, true)
                .as_str(),
        );

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
            self.calc_button_click();

            let mut model = if let Some(model) = self.model.take() {
                model
            } else {
                SecurityModel::default()
            };

            if let Ok(id) = self.security_form_ui.id_input.text().parse() {
                model.id = id;
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
            model.time = Utc::now();

            if let Ok(conn) = Connection::open("./water-resources.db") {
                if let Ok(num) = if model.id > 0 {
                    conn.execute(
                    r#"UPDATE water_security SET
                        level=?1, name=?2, area=?3, start=?4, end=?5, river_width=?6, elevation=?7, ratio=?8, 
line=?9, allow=?10, safe=?11, depth=?12, channel_width=?13, threshold=?14, dredging=?15, time=?16
WHERE id=?17
            "#,
                        params![
                            model.level,
                            model.name,
                            model.area,
                            model.start,
                            model.end,
                            model.river_width,
                            model.elevation,
                            model.ratio,
                            model.line,
                            model.allow,
                            model.safe,
                            model.depth,
                            model.channel_width,
                            model.threshold,
                            model.dredging,
                            model.time,
                            model.id
                        ],
                    )
                } else {
                    conn.execute(
                    r#"INSERT INTO 
water_security(level, name, area, start, end, river_width, elevation, ratio, line, allow, safe, depth, channel_width, threshold, dredging, time) 
VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)"#,
                        params![
                            model.level,
                            model.name,
                            model.area,
                            model.start,
                            model.end,
                            model.river_width,
                            model.elevation,
                            model.ratio,
                            model.line,
                            model.allow,
                            model.safe,
                            model.depth,
                            model.channel_width,
                            model.threshold,
                            model.dredging,
                            model.time
                        ],
                    )
                } {
                    if num == 1 {
                        nwg::simple_message("提示", "保存成功");
                    } else {
                        nwg::simple_message("提示", "保存失败");
                    }
                } else {
                    nwg::simple_message("提示", "保存失败");
                }
            }

            *self.model.borrow_mut() = Some(model);

            self.window.close();
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
    // TODO RefCell指针改为Box指针
    security_models: RefCell<Option<Vec<SecurityModel>>>,
    security_window_handle: RefCell<Option<JoinHandle<SecurityModel>>>,

    #[nwg_control(size: (900, 600), center: true, title: "水安全", flags: "MAIN_WINDOW | VISIBLE")]
    #[nwg_events(OnWindowClose: [Self::window_close], OnInit: [Self::init_data_view])]
    window: nwg::Window,

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
                    stmt.query_and_then([], |row| -> rusqlite::Result<SecurityModel> {
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
                    let mut models = vec![];
                    while let Some(Ok(model)) = security_models.next() {
                        models.push(model.clone());

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
                                model.time.to_rfc3339_opts(SecondsFormat::Secs, true),
                            ],
                        );
                    }
                    *self.security_models.borrow_mut() = Some(models);
                }
            }
        }
    }

    // TODO 数据导入功能
    fn import_menu_open(&self) {
        nwg::simple_message("导入", "数据导入功能");
    }

    // TODO 数据导出功能
    fn export_menu_open(&self) {
        nwg::simple_message("导出", "数据导出功能");
    }

    fn create_menu_open(&self) {
        *self.security_window_handle.borrow_mut() = Some(SecurityFormWindow::window_open(None));

        self.window.set_visible(false);

        if let Some(mut models) = self.security_models.take() {
            let handle = self.security_window_handle.borrow_mut().take();
            if let Some(handle) = handle {
                if let Ok(model) = handle.join() {
                    models.push(model);
                    *self.security_models.borrow_mut() = Some(models);

                    self.reload_menu_selected();

                    self.window.set_visible(true);
                }
            }
        }
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
            if let Some(mut models) = self.security_models.take() {
                if index < models.len() {
                    let model = models[index].clone();
                    *self.security_window_handle.borrow_mut() =
                        Some(SecurityFormWindow::window_open(Some(model)));

                    self.window.set_visible(false);

                    let handle = self.security_window_handle.borrow_mut().take();
                    if let Some(handle) = handle {
                        if let Ok(model) = handle.join() {
                            models[index] = model;
                            *self.security_models.borrow_mut() = Some(models);

                            self.reload_menu_selected();

                            self.window.set_visible(true);
                        }
                    }
                }
            }
        }
    }

    fn delete_menu_selected(&self) {
        if let Some(index) = self.data_view.selected_item() {
            if let Some(models) = self.security_models.take() {
                if index < models.len() {
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
                        let id = models[index].id;

                        if let Ok(conn) = Connection::open("./water-resources.db") {
                            if let Ok(num) =
                                conn.execute("DELETE FROM water_security WHERE id=?", [id])
                            {
                                if num == 1 {
                                    nwg::simple_message("提示", "删除成功");
                                } else {
                                    nwg::simple_message("提示", "删除失败");
                                }
                            } else {
                                nwg::simple_message("提示", "删除失败");
                            }
                        }
                    }

                    *self.security_models.borrow_mut() = Some(models);

                    self.reload_menu_selected();
                }
            }
        }
    }
}
