use chrono::{DateTime, SecondsFormat, Utc};
use rusqlite::{params_from_iter, ParamsFromIter, Result};

use crate::db::{DbStatement, Model};

#[derive(Debug, Clone)]
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
    pub time: DateTime<Utc>,
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

impl<'a> Model<'a> for SecurityModel {
    fn from_row(row: &rusqlite::Row<'_>) -> Result<Self> {
        Ok(SecurityModel {
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

    fn get_sql(db_stmt: DbStatement) -> &'a str {
        match db_stmt {
            DbStatement::Create => {
                r#"CREATE TABLE water_security
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
            }
            DbStatement::Insert => {
                r#"INSERT INTO water_security(
                    level, name, area, start, end, river_width, elevation, ratio, 
                    line, allow, safe, depth, channel_width, threshold, dredging, time
                ) 
                VALUES(
                    ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16
                )"#
            }
            DbStatement::Update => {
                r#"UPDATE water_security SET
                level=?1, name=?2, area=?3, start=?4, end=?5, river_width=?6, elevation=?7, ratio=?8, 
                line=?9, allow=?10, safe=?11, depth=?12, channel_width=?13, threshold=?14, dredging=?15, time=?16
                WHERE id=?17"#
            }
            DbStatement::Delete => r#"DELETE FROM water_security WHERE id=?"#,
            DbStatement::Select => {
                r#"SELECT 
                id, level, name, area, start, end, river_width, elevation, ratio, 
                line, allow, safe, depth, channel_width, threshold, dredging, time 
                FROM water_security"#
            }
        }
    }

    // TODO 参数绑定对应类型
    fn get_params(&self, db_stmt: DbStatement) -> ParamsFromIter<Vec<String>> {
        let model = self.clone();
        match db_stmt {
            DbStatement::Create => params_from_iter(vec![]),
            DbStatement::Insert => params_from_iter(vec![
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
                model.time.to_rfc3339_opts(SecondsFormat::Secs, true),
            ]),
            DbStatement::Update => params_from_iter(vec![
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
                model.time.to_rfc3339_opts(SecondsFormat::Secs, true),
                model.id.to_string(),
            ]),
            DbStatement::Delete => params_from_iter(vec![model.id.to_string()]),
            DbStatement::Select => params_from_iter(vec![]),
        }
    }
}
