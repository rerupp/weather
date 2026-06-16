//! The country table definition.
use super::{create_insert_sql, TblSqlBuilder};

pub const SCHEMA: &str = r"
CREATE TABLE IF NOT EXISTS country
(
    id   INTEGER PRIMARY KEY,
    name TEXT NOT NULL COLLATE nocase,
    code TEXT NOT NULL COLLATE nocase,
    UNIQUE (name, code)
);
CREATE INDEX IF NOT EXISTS idx_country_name on country (name COLLATE nocase);
CREATE INDEX IF NOT EXISTS idx_country_code on country (code COLLATE nocase);
";

/// The cities database Country table
///
#[derive(Debug)]
pub enum CountryTbl {
    // the enum must follow the schema column order
    Id,
    Name,
    Code,
}
impl CountryTbl {
    // the array MUST be in enum order to stay in sync
    const COLUMN_PARAM: [(&str, &str); 3] = [("id", ":id"), ("name", ":name"), ("code", ":code")];

    /// The schema name for the table.
    pub const TABLE: &str = "country";

    /// Generate the SQL fragment '`country AS alias`'.
    ///
    /// # Arguments
    ///
    /// * `alias` is the table alias name.
    ///
    pub fn table_as(alias: &str) -> String {
        format!("{} AS {}", Self::TABLE, alias.to_string())
    }

    /// Get the SQL that will insert a row into the table.
    ///
    pub fn insert_sql() -> String {
        create_insert_sql(Self::TABLE, &Self::COLUMN_PARAM)
    }
}
impl TblSqlBuilder for CountryTbl {
    /// The column names.
    ///
    fn column(&self) -> &'static str {
        match self {
            Self::Id => Self::COLUMN_PARAM[Self::Id as usize].0,
            Self::Name => Self::COLUMN_PARAM[Self::Name as usize].0,
            Self::Code => Self::COLUMN_PARAM[Self::Code as usize].0,
        }
    }

    /// The column parameter names.
    ///
    fn param(&self) -> &'static str {
        match self {
            Self::Id => Self::COLUMN_PARAM[Self::Id as usize].1,
            Self::Name => Self::COLUMN_PARAM[Self::Name as usize].1,
            Self::Code => Self::COLUMN_PARAM[Self::Code as usize].1,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::backend::db::sqlite::{
        db_connection, err, execute_sql, prepare_cached_sql, prepare_sql,
        tables::{cities, named_param},
    };
    use rusqlite::Connection;
    use sql_query_builder as sql;

    // a helper for testcases that will need to insert a country
    pub fn insert_country(conn: &mut Connection, name: &str, code: &str) -> i64 {
        let insert_sql = CountryTbl::insert_sql();
        let mut stmt = prepare_sql!(conn, &insert_sql, "failed to prepare country insert SQL").unwrap();
        let params = [named_param!(CountryTbl::Name, name), named_param!(CountryTbl::Code, code)];
        execute_sql!(stmt, &params, "failed to insert country").unwrap();
        conn.last_insert_rowid()
    }

    #[test]
    fn row_insert() {
        // use an in-memory database
        let mut conn = db_connection(None).unwrap();
        cities::initialize_schema(&conn).unwrap();

        let country_name = "Country";
        let country_code = "CO";
        insert_country(&mut conn, country_name, country_code);

        // verify the row content
        let query_sql = sql::Select::new()
            .select(CountryTbl::Name.column())
            .select(CountryTbl::Code.column())
            .from(CountryTbl::TABLE)
            .to_string();
        let mut stmt = prepare_cached_sql!(conn, &query_sql, "failed to prepare country verify SQL").unwrap();
        stmt.query_one([], |row| {
            assert_eq!(row.get::<_, String>(CountryTbl::Name.column()).unwrap(), country_name);
            assert_eq!(row.get::<_, String>(CountryTbl::Code.column()).unwrap(), country_code);
            Ok(())
        })
        .unwrap();
    }
}
