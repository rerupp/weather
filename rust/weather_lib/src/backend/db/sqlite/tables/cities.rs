//! The weather history database tables.

use super::{create_insert_sql, TblSqlBuilder};
use rusqlite::Connection;
use sql_query_builder as sql;

mod city;
pub use city::CityTbl;

mod country;
pub use country::CountryTbl;

mod region;
pub use region::RegionTbl;

#[doc(hidden)]
macro_rules! err {
    ($($arg:tt)*) => {
        Err(crate::Error(format!("SQLite cities schema: {}", format!($($arg)*))))
    };
}

/// Initialize the cities database schema.
///
/// # Arguments
///
/// * `conn` is the database connection that will be used.
///
pub fn initialize_schema(conn: &Connection) -> crate::Result<()> {
    // limit the scope of writing to strings
    use std::fmt::Write;
    let mut sql = String::new();
    writeln!(sql, "BEGIN;").unwrap();
    writeln!(sql, "{}", country::SCHEMA).unwrap();
    writeln!(sql, "{}", region::SCHEMA).unwrap();
    writeln!(sql, "{}", city::SCHEMA).unwrap();
    writeln!(sql, "COMMIT;").unwrap();
    match conn.execute_batch(&sql) {
        Ok(_) => Ok(()),
        Err(error) => err!("failed to initialize history schema.\n{error:?}")?,
    }
}

/// Drop the cities database schema.
///
/// # Arguments
///
/// * `conn` is the database connection that will be used.
///
pub fn drop_schema(conn: &Connection) -> crate::Result<()> {
    // limit the scope of writing to strings
    use std::fmt::Write;
    let mut sql = String::new();
    writeln!(sql, "BEGIN;").unwrap();
    let drop = |table| format!("DROP TABLE IF EXISTS {table};");
    writeln!(sql, "{}", drop(CityTbl::TABLE)).unwrap();
    writeln!(sql, "{}", drop(RegionTbl::TABLE)).unwrap();
    writeln!(sql, "{}", drop(CountryTbl::TABLE)).unwrap();
    writeln!(sql, "COMMIT;").unwrap();
    match conn.execute_batch(&sql) {
        Ok(_) => Ok(()),
        Err(error) => err!("failed to drop history schema.\n{error:?}"),
    }
}

///  Find out if the cities database schema has been initialized.
///
/// # Arguments
///
/// * `conn` is the database connection that will be used.
///
pub fn is_schema_initialized(conn: &Connection) -> crate::Result<bool> {
    use crate::backend::db::sqlite::prepare_sql;
    let table_match = |name| format!("name='{name}'");
    let tables_query = sql::Select::new()
        .select("COUNT(*)")
        .from("pragma_table_list")
        .where_or(&table_match(CountryTbl::TABLE))
        .where_or(&table_match(RegionTbl::TABLE))
        .where_or(&table_match(CityTbl::TABLE))
        .to_string();
    let mut stmt = prepare_sql!(conn, &tables_query, "failed to prepare is_initialized query")?;
    let count = stmt.query_one([], |row| Ok(row.get::<_, i64>(0).unwrap())).unwrap();
    Ok(count == 3)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::db::sqlite::db_connection;

    #[test]
    fn initialize_drop() {
        // use an in-memory database for the tests
        let conn = db_connection(None).unwrap();
        assert!(!is_schema_initialized(&conn).unwrap());
        initialize_schema(&conn).unwrap();
        assert!(is_schema_initialized(&conn).unwrap());
        drop_schema(&conn).unwrap();
        assert!(!is_schema_initialized(&conn).unwrap());
    }
}
