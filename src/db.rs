use std::borrow::Borrow;

use rusqlite::{Connection, Result, Row, Statement, ToSql};

pub enum DbOpt {
    Create,
    Insert,
    Update,
    Delete,
    Select,
}

pub struct DbConn<T: Model> {
    pub instance: Box<Connection>,
    pub model: Box<Option<T>>,
}

impl<T: Model> DbConn<T> {
    pub fn new() -> Self {
        Self {
            instance: Box::new(
                Connection::open("./water-resources.db").expect("Connect database failed."),
            ),
            model: Box::new(None),
        }
    }
    pub fn prepare(&self, sql: &str) -> Result<Statement<'_>> {
        self.instance.prepare(sql)
    }
    pub fn create(&self) -> Result<usize> {
        self.prepare(T::get_sql(DbOpt::Create).as_str())?
            .execute([])
    }
    pub fn insert(&self) -> Result<usize> {
        let mut stmt = self.prepare(T::get_sql(DbOpt::Insert).as_str())?;
        if let Some(model) = self.model.borrow() {
            model.execute(&mut stmt, DbOpt::Insert)
        } else {
            Ok(0)
        }
    }
    pub fn update(&self) -> Result<usize> {
        let mut stmt = self.prepare(T::get_sql(DbOpt::Update).as_str())?;
        if let Some(model) = self.model.borrow() {
            model.execute(&mut stmt, DbOpt::Update)
        } else {
            Ok(0)
        }
    }
    pub fn delete(&self) -> Result<usize> {
        let mut stmt = self.prepare(T::get_sql(DbOpt::Delete).as_str())?;
        if let Some(model) = self.model.borrow() {
            model.execute(&mut stmt, DbOpt::Delete)
        } else {
            Ok(0)
        }
    }
    pub fn select(&mut self) -> Result<Vec<T>> {
        let mut stmt = self.prepare(T::get_sql(DbOpt::Select).as_str())?;
        let models = stmt.query_and_then([], |row| -> Result<T> { T::from_row(row) })?;
        let mut result_models = Vec::new();
        for model in models {
            result_models.push(model?);
        }
        Ok(result_models)
    }
    pub fn find(
        &self,
        condition: &str,
        order: (&str, &str),
        limit: (u32, u32),
        params: &[(&str, &dyn ToSql)],
    ) -> Result<Vec<T>> {
        let mut stmt = self.prepare(T::get_sql_with_condition(condition, order, limit).as_str())?;
        let mut rows = stmt.query(params)?;
        let mut models = Vec::new();
        while let Some(row) = rows.next()? {
            let model = T::from_row(row)?;
            models.push(model);
        }
        Ok(models)
    }
    pub fn find_first(
        &self,
        condition: &str,
        order: (&str, &str),
        limit: (u32, u32),
        params: &[(&str, &dyn ToSql)],
    ) -> Result<T> {
        let mut stmt = self.prepare(T::get_sql_with_condition(condition, order, limit).as_str())?;
        stmt.query_row(params, |row| T::from_row(row))
    }
    pub fn find_by_id(&self, id: u32) -> Result<T> {
        let mut stmt =
            self.prepare(format!("{} WHERE id=?", T::get_sql(DbOpt::Select)).as_str())?;
        stmt.query_row([id], |row| T::from_row(row))
    }
}

pub enum ModelNameType {
    Column,
    Header,
}

pub trait Model<T = Self> {
    // TODO 使用HeaderType代替手动设置
    fn get_names(name_type: ModelNameType) -> Vec<String>;
    fn get_sql(opt: DbOpt) -> String;
    fn get_sql_with_condition(condition: &str, order: (&str, &str), limit: (u32, u32)) -> String {
        let mut condition = condition.trim_start().to_string();
        if condition.len() < 5 {
            condition = String::from(" WHERE 1=1 ");
        }
        if condition[..5].to_uppercase() != "WHERE" {
            condition = format!(" WHERE {} ", condition);
        }
        if !condition.to_uppercase().contains("ORDER BY")
            && Self::get_names(ModelNameType::Column).contains(&String::from(order.0))
        {
            condition = format!(
                " {} ORDER BY {} {} ",
                condition,
                order.0,
                if order.1 == "DESC" { "DESC" } else { "ASC" }
            );
        }
        if limit != (0u32, 0u32) {
            condition = format!(" {} LIMIT {} OFFSET {} ", condition, limit.0, limit.1);
        }
        format!("{} {}", Self::get_sql(DbOpt::Select), condition)
    }
    fn from_row(row: &Row) -> Result<T>;
    fn execute(&self, stmt: &mut Statement, opt: DbOpt) -> Result<usize>;
}
