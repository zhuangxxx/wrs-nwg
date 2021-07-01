use rusqlite::{named_params, Connection, ParamsFromIter, Result, Row, ToSql};

pub trait Model<'a, T = Self> {
    fn from_row(row: &Row<'_>) -> Result<T>;
    fn get_sql(db_stmt: DbStatement) -> &'a str;
    fn get_params(&self, db_stmt: DbStatement) -> ParamsFromIter<Vec<String>>;
}

pub enum DbStatement {
    Create,
    Insert,
    Update,
    Delete,
    Select,
}

pub struct Db {
    pub connection: Box<Connection>,
}

impl Db {
    pub fn new() -> Self {
        Self {
            connection: Box::new(
                Connection::open("./water-resources.db").expect("database connect failed"),
            ),
        }
    }
    pub fn connect(&mut self, path: &str) -> Result<()> {
        self.connection = Box::new(Connection::open(path)?);
        Ok(())
    }
    pub fn create<'a, T: Model<'a>>(&self) -> Result<usize> {
        self.connection.execute(T::get_sql(DbStatement::Create), [])
    }
    pub fn insert<'a, T: Model<'a>>(&self, model: T) -> Result<usize> {
        self.connection.execute(
            T::get_sql(DbStatement::Insert),
            model.get_params(DbStatement::Insert),
        )
    }
    pub fn update<'a, T: Model<'a>>(&self, model: T) -> Result<usize> {
        self.connection.execute(
            T::get_sql(DbStatement::Update),
            model.get_params(DbStatement::Update),
        )
    }
    pub fn delete<'a, T: Model<'a>>(&self, model: T) -> Result<usize> {
        self.connection.execute(
            T::get_sql(DbStatement::Delete),
            model.get_params(DbStatement::Delete),
        )
    }
    pub fn select<'a, T: Model<'a>>(&self) -> Result<Vec<T>> {
        let mut stmt = self.connection.prepare(T::get_sql(DbStatement::Select))?;
        let rows = stmt.query_and_then([], |row| T::from_row(row))?;
        let mut models = vec![];
        for model in rows {
            models.push(model?);
        }
        Ok(models)
    }
    pub fn find<'a, T: Model<'a>>(
        &self,
        sql: &str,
        params: &[(&str, &dyn ToSql)],
    ) -> Result<Vec<T>> {
        let mut stmt = self.connection.prepare(sql)?;
        let rows = stmt.query_and_then(params, |row| T::from_row(row))?;
        let mut models = vec![];
        for model in rows {
            models.push(model?);
        }
        Ok(models)
    }
    pub fn find_one<'a, T: Model<'a>>(
        &self,
        sql: &str,
        params: &[(&str, &dyn ToSql)],
    ) -> Result<T> {
        self.connection
            .query_row_and_then(sql, params, |row| T::from_row(row))
    }
    pub fn find_by_id<'a, T: Model<'a>>(&self, sql: &str, id: u32) -> Result<T> {
        self.find_one(sql, named_params! {":id":id})
    }
}
