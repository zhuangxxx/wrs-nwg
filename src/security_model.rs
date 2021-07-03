use chrono::{DateTime, Local};
use rusqlite::{named_params, Result, Row, Statement};

use crate::db::{DbOpt, Model, ModelNameType};

#[derive(Clone)]
pub struct SecurityModel {
    pub id: u32,
    pub level: u32,
    pub name: String,
    pub area: String,
    pub start: String,
    pub end: String,
    pub river_width: f32,
    pub ratio: f32,
    pub elevation: f32,
    pub line: f32,
    pub allow: f32,
    pub safe: f32,
    pub depth: f32,
    pub channel_width: f32,
    pub threshold: f32,
    pub dredging: String,
    pub time: DateTime<Local>,
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
            time: Local::now(),
        }
    }
}

impl Model for SecurityModel {
    fn get_sql(opt: DbOpt) -> String {
        match opt {
            DbOpt::Create => r#"CREATE TABLE water_security
                (
                    id            INTEGER PRIMARY KEY AUTOINCREMENT,
                    level         INTEGER,
                    name          TEXT,
                    area          TEXT,
                    start         TEXT,
                    end           TEXT,
                    river_width   REAL,
                    elevation     REAL,
                    ratio         REAL,
                    line          REAL,
                    allow         REAL,
                    safe          REAL,
                    depth         REAL,
                    channel_width REAL,
                    threshold     REAL,
                    dredging      TEXT,
                    time          TEXT
                )"#
            .to_string(),
            DbOpt::Insert => r#"INSERT INTO water_security(
                    level, name, area, start, end, river_width, elevation, ratio,
                    line, allow, safe, depth, channel_width, threshold, dredging, time
                )
                VALUES(
                    :level, :name, :area, :start, :end, :river_width, :elevation, :ratio,
                    :line, :allow, :safe, :depth, :channel_width, :threshold, :dredging, :time
                )"#
            .to_string(),
            DbOpt::Update => r#"UPDATE water_security SET
                    level=:level, name=:name, area=:area, start=:start, end=:end,
                    river_width=:river_width, elevation=:elevation, ratio=:ratio,
                    line=:line, allow=:allow, safe=:safe, depth=:depth, channel_width=:channel_width,
                    threshold=:threshold, dredging=:dredging, time=:time
                WHERE id=:id"#
                .to_string(),
            DbOpt::Delete => r#"DELETE FROM water_security WHERE id=?"#.to_string(),
            DbOpt::Select => r#"SELECT
                id, level, name, area, start, end, river_width, elevation, ratio,
                line, allow, safe, depth, channel_width, threshold, dredging, time
                FROM water_security"#
                .to_string(),
        }
    }

    fn get_names(name_type: ModelNameType) -> Vec<String> {
        match name_type {
            ModelNameType::Column => vec![
                String::from("id"),
                String::from("level"),
                String::from("name"),
                String::from("area"),
                String::from("start"),
                String::from("end"),
                String::from("river_width"),
                String::from("elevation"),
                String::from("ratio"),
                String::from("line"),
                String::from("allow"),
                String::from("safe"),
                String::from("depth"),
                String::from("channel_width"),
                String::from("threshold"),
                String::from("dredging"),
                String::from("time"),
            ],
            ModelNameType::Header => todo!(),
        }
    }

    fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            level: row.get("level")?,
            name: row.get("name")?,
            area: row.get("area")?,
            start: row.get("start")?,
            end: row.get("end")?,
            river_width: row.get("river_width")?,
            elevation: row.get("elevation")?,
            ratio: row.get("ratio")?,
            line: row.get("line")?,
            allow: row.get("allow")?,
            safe: row.get("safe")?,
            depth: row.get("depth")?,
            channel_width: row.get("channel_width")?,
            threshold: row.get("threshold")?,
            dredging: row.get("dredging")?,
            time: row.get("time")?,
        })
    }

    fn execute(&self, stmt: &mut Statement, opt: DbOpt) -> Result<usize> {
        match opt {
            DbOpt::Create => unimplemented!(),
            DbOpt::Insert => stmt.execute(named_params! {
                ":level": self.level,
                ":name": self.name,
                ":area": self.area,
                ":start": self.start,
                ":end": self.end,
                ":river_width": self.river_width,
                ":ratio": self.ratio,
                ":elevation": self.elevation,
                ":line": self.line,
                ":allow": self.allow,
                ":safe": self.safe,
                ":depth": self.depth,
                ":channel_width": self.channel_width,
                ":threshold": self.threshold,
                ":dredging": self.dredging,
                ":time": self.time,
            }),
            DbOpt::Update => stmt.execute(named_params! {
                ":level": self.level,
                ":name": self.name,
                ":area": self.area,
                ":start": self.start,
                ":end": self.end,
                ":river_width": self.river_width,
                ":ratio": self.ratio,
                ":elevation": self.elevation,
                ":line": self.line,
                ":allow": self.allow,
                ":safe": self.safe,
                ":depth": self.depth,
                ":channel_width": self.channel_width,
                ":threshold": self.threshold,
                ":dredging": self.dredging,
                ":time": self.time,
                ":id": self.id,
            }),
            DbOpt::Delete => stmt.execute([self.id]),
            DbOpt::Select => unimplemented!(),
        }
    }
}
