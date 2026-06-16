//! The weather history database tables.

use super::{create_insert_sql, TblSqlBuilder};
use rusqlite::Connection;
use sql_query_builder as sql;

mod locations;
pub use locations::LocationsTbl;

mod dates;
pub use dates::DatesTbl;

mod metadata;
pub use metadata::MetadataTbl;

mod history;
use crate::backend::db::sqlite::prepare_cached_sql;
pub use history::HistoryTbl;

#[doc(hidden)]
macro_rules! err {
    ($($arg:tt)*) => {
        Err(crate::Error(format!("SQLite weather schema: {}", format!($($arg)*))))
    };
}

/// Initialize the weather history database schema.
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
    writeln!(sql, "{}", locations::SCHEMA).unwrap();
    writeln!(sql, "{}", dates::SCHEMA).unwrap();
    writeln!(sql, "{}", metadata::SCHEMA).unwrap();
    writeln!(sql, "{}", history::SCHEMA).unwrap();
    writeln!(sql, "COMMIT;").unwrap();
    match conn.execute_batch(&sql) {
        Ok(_) => Ok(()),
        Err(error) => err!("failed to initialize history schema.\n{error:?}")?,
    }
}

/// Drop the weather history database schema.
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
    writeln!(sql, "{}", drop(HistoryTbl::TABLE)).unwrap();
    writeln!(sql, "{}", drop(MetadataTbl::TABLE)).unwrap();
    writeln!(sql, "{}", drop(DatesTbl::TABLE)).unwrap();
    writeln!(sql, "{}", drop(LocationsTbl::TABLE)).unwrap();
    writeln!(sql, "COMMIT;").unwrap();
    match conn.execute_batch(&sql) {
        Ok(_) => Ok(()),
        Err(error) => err!("failed to drop history schema.\n{error:?}"),
    }
}

/// Find out if the weather history database schema has been initialized.
///
/// # Arguments
///
/// * `conn` is the database connection that will be used.
///
pub fn is_schema_initialized(conn: &Connection) -> crate::Result<bool> {
    let table_match = |name| format!("name='{name}'");
    let tables_query = sql::Select::new()
        .select("COUNT(*)")
        .from("pragma_table_list")
        .where_or(&table_match(LocationsTbl::TABLE))
        .where_or(&table_match(DatesTbl::TABLE))
        .where_or(&table_match(MetadataTbl::TABLE))
        .where_or(&table_match(HistoryTbl::TABLE))
        .to_string();
    let mut stmt = prepare_cached_sql!(conn, &tables_query, "failed to prepare is_initialized query")?;
    let count = stmt.query_one([], |row| Ok(row.get::<_, i64>(0).unwrap())).unwrap();
    Ok(count == 4)
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
