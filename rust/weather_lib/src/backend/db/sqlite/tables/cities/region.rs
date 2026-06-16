//! The region table definition.
use super::{create_insert_sql, CountryTbl, TblSqlBuilder};

pub const SCHEMA: &str = r"
CREATE TABLE IF NOT EXISTS region
(
    id   INTEGER PRIMARY KEY,
    cid  INTEGER,
    name TEXT NOT NULL COLLATE nocase,
    code TEXT NOT NULL COLLATE nocase,
    UNIQUE (cid, name, code),
    -- back link to the country
    FOREIGN KEY (cid) REFERENCES country (id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_region_coid on region (cid);
CREATE INDEX IF NOT EXISTS idx_region_name on region (name COLLATE nocase);
CREATE INDEX IF NOT EXISTS idx_region_code on region (code COLLATE nocase);
";

/// The cities database Country table
///
#[derive(Debug)]
pub enum RegionTbl {
    // the enum must follow the schema column order
    Id,
    Cid,
    Name,
    Code,
}
impl RegionTbl {
    // the array MUST be in enum order to stay in sync
    const COLUMN_PARAM: [(&str, &str); 4] = [("id", ":id"), ("cid", ":cid"), ("name", ":name"), ("code", ":code")];

    /// The schema name for the table.
    pub const TABLE: &str = "region";

    /// Generate the SQL fragment '`region AS alias`'.
    ///
    /// # Arguments
    ///
    /// * `alias` is the table alias name.
    ///
    pub fn table_as(alias: &str) -> String {
        format!("{} AS {}", Self::TABLE, alias.to_string())
    }

    /// Generate the SQL fragment '`region AS r co.id=r.rid`'.
    ///
    /// # Arguments
    ///
    /// * `r` is the region table alias name.
    /// * `co` is the country table alias name.
    ///
    pub fn alias_join_country_as(r: &str, co: &str) -> String {
        format!("{} ON {}={}", RegionTbl::table_as(r), CountryTbl::Id.alias_column(co), RegionTbl::Cid.alias_column(r))
    }

    /// Get the SQL that will insert a row into the table.
    ///
    pub fn insert_sql() -> String {
        create_insert_sql(Self::TABLE, &Self::COLUMN_PARAM)
    }
}
impl TblSqlBuilder for RegionTbl {
    /// The column names.
    ///
    fn column(&self) -> &'static str {
        match self {
            Self::Id => Self::COLUMN_PARAM[Self::Id as usize].0,
            Self::Cid => Self::COLUMN_PARAM[Self::Cid as usize].0,
            Self::Name => Self::COLUMN_PARAM[Self::Name as usize].0,
            Self::Code => Self::COLUMN_PARAM[Self::Code as usize].0,
        }
    }

    /// The column parameter names.
    ///
    fn param(&self) -> &'static str {
        match self {
            Self::Id => Self::COLUMN_PARAM[Self::Id as usize].1,
            Self::Cid => Self::COLUMN_PARAM[Self::Cid as usize].1,
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
    pub fn insert_region(conn: &mut Connection, name: &str, code: &str, cid: i64) -> i64 {
        let insert_sql = RegionTbl::insert_sql();
        let mut stmt = prepare_sql!(conn, &insert_sql, "failed to prepare region insert SQL").unwrap();
        let params = [
            named_param!(RegionTbl::Cid, cid),
            named_param!(RegionTbl::Name, name),
            named_param!(RegionTbl::Code, code),
        ];
        execute_sql!(stmt, &params, "failed to insert country").unwrap();
        conn.last_insert_rowid()
    }

    #[test]
    fn row_insert() {
        // use an in-memory database
        let mut conn = db_connection(None).unwrap();
        cities::initialize_schema(&conn).unwrap();

        // setup the test environment
        let cid = cities::country::tests::insert_country(&mut conn, "", "");

        let region_name = "Region";
        let region_code = "RE";
        insert_region(&mut conn, region_name, region_code, cid);

        // verify the row content
        let query_sql = sql::Select::new()
            .select(RegionTbl::Cid.column())
            .select(RegionTbl::Name.column())
            .select(RegionTbl::Code.column())
            .from(RegionTbl::TABLE)
            .to_string();
        let mut stmt = prepare_cached_sql!(conn, &query_sql, "failed to prepare region verify SQL").unwrap();
        stmt.query_one([], |row| {
            assert_eq!(row.get::<_, i64>(RegionTbl::Cid.column()).unwrap(), cid);
            assert_eq!(row.get::<_, String>(RegionTbl::Name.column()).unwrap(), region_name);
            assert_eq!(row.get::<_, String>(RegionTbl::Code.column()).unwrap(), region_code);
            Ok(())
        })
        .unwrap();
    }
}
