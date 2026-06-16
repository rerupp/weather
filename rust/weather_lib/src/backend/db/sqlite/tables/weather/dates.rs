//! The dates table definition.
use super::{create_insert_sql, LocationsTbl, TblSqlBuilder};

pub const SCHEMA: &'static str = r"
CREATE TABLE IF NOT EXISTS dates
(
    id   INTEGER PRIMARY KEY,
    lid  INTEGER NOT NULL,
    date TEXT    NOT NULL,
    -- backlink to the associated location
    FOREIGN KEY (lid) REFERENCES locations (id),
    CONSTRAINT uc_dates_lid_date UNIQUE (lid, date)
);
CREATE INDEX IF NOT EXISTS idx_dates_lid on dates (lid);
CREATE INDEX IF NOT EXISTS idx_dates_date on dates (date);
";

#[derive(Debug)]
pub enum DatesTbl {
    // the enum must follow the schema column order
    Id,
    Lid,
    Date,
}
impl DatesTbl {
    // the array MUST be in enum order to stay in sync
    const COLUMN_PARAM: [(&str, &str); 3] = [("id", ":id"), ("lid", ":lid"), ("date", ":date")];

    /// The schema table name.
    pub const TABLE: &str = "dates";

    /// Generate the SQL fragment '`table AS alias`'.
    ///
    /// # Arguments
    ///
    /// * `alias` is the table alias name.
    ///
    pub fn table_as(alias: impl ToString) -> String {
        format!("{} AS {}", Self::TABLE, alias.to_string())
    }

    /// Generate the SQL fragment '`dates AS d l.id=d.lid`'.
    ///
    /// # Arguments
    ///
    /// * `d` is the dates table alias name.
    /// * `l` is the locations table alias name.
    ///
    pub fn alias_join_locations_as(d: &str, l: &str) -> String {
        format!("{} ON {}={}", DatesTbl::table_as(d), LocationsTbl::Id.alias_column(l), DatesTbl::Lid.alias_column(d))
    }

    /// Generate the SQL that will insert a row into the table.
    ///
    pub fn insert_sql() -> String {
        create_insert_sql(Self::TABLE, &Self::COLUMN_PARAM)
    }
}
impl TblSqlBuilder for DatesTbl {
    /// Get the column name.
    ///
    fn column(&self) -> &'static str {
        match self {
            Self::Id => Self::COLUMN_PARAM[Self::Id as usize].0,
            Self::Lid => Self::COLUMN_PARAM[Self::Lid as usize].0,
            Self::Date => Self::COLUMN_PARAM[Self::Date as usize].0,
        }
    }

    /// The column parameters.
    ///
    fn param(&self) -> &'static str {
        match self {
            Self::Id => Self::COLUMN_PARAM[Self::Id as usize].1,
            Self::Lid => Self::COLUMN_PARAM[Self::Lid as usize].1,
            Self::Date => Self::COLUMN_PARAM[Self::Date as usize].1,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{
        backend::db::sqlite::{
            db_connection, err, execute_sql, prepare_cached_sql, prepare_sql,
            tables::{named_param, weather},
        },
        entities::Location,
    };
    use chrono::NaiveDate;
    use rusqlite::Connection;
    use sql_query_builder as sql;

    pub fn insert_date(conn: &mut Connection, lid: i64, date: NaiveDate) -> i64 {
        let insert_sql = DatesTbl::insert_sql();
        let mut stmt = prepare_sql!(conn, &insert_sql, "failed to prepare dates insert SQL").unwrap();
        let params = [named_param!(DatesTbl::Lid, lid), named_param!(DatesTbl::Date, date)];
        execute_sql!(stmt, &params, "failed to insert dates").unwrap();
        conn.last_insert_rowid()
    }

    #[test]
    fn row_insert() {
        // use an in-memory database
        let mut conn = db_connection(None).unwrap();
        weather::initialize_schema(&conn).unwrap();

        // add content
        let location = Location { alias: "alias".to_string(), ..Default::default() };
        let lid = weather::locations::tests::insert_location(&mut conn, location);

        let date = NaiveDate::from_ymd_opt(2026, 6, 9).unwrap();
        insert_date(&mut conn, lid, date);

        let query_sql = sql::Select::new()
            .select(DatesTbl::Lid.column())
            .select(DatesTbl::Date.column())
            .from(DatesTbl::TABLE)
            .to_string();
        let mut stmt = prepare_cached_sql!(conn, &query_sql, "failed to prepare dates query SQL").unwrap();
        stmt.query_one([], |row| {
            assert_eq!(row.get::<_, i64>(DatesTbl::Lid.column()).unwrap(), lid);
            assert_eq!(row.get::<_, NaiveDate>(DatesTbl::Date.column()).unwrap(), date);
            Ok(())
        })
        .unwrap()
    }
}
