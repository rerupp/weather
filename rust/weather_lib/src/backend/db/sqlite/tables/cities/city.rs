//! The cities table definition.
use super::{create_insert_sql, region::RegionTbl, TblSqlBuilder};

pub const SCHEMA: &str = r"
CREATE TABLE IF NOT EXISTS city
(
    id   INTEGER PRIMARY KEY,
    rid  INTEGER,
    name TEXT NOT NULL COLLATE nocase,
    lat  TEXT NOT NULL,
    lng  TEXT NOT NULL,
    tz   TEXT NOT NULL COLLATE nocase,
    -- back link to the region
    FOREIGN KEY (rid) REFERENCES region (id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_city_rid ON city (rid);
CREATE INDEX IF NOT EXISTS idx_city_name ON city (name COLLATE nocase);
";

/// The cities database Country table
///
#[derive(Debug)]
pub enum CityTbl {
    // the enum must follow the schema column order
    Id,
    Rid,
    Name,
    Latitude,
    Longitude,
    Tz,
}
impl CityTbl {
    // the array MUST be in enum order to stay in sync
    const COLUMN_PARAM: [(&str, &str); 6] =
        [("id", ":id"), ("rid", ":rid"), ("name", ":name"), ("lat", ":lat"), ("lng", ":lng"), ("tz", ":tz")];

    /// The schema name for the table.
    ///
    pub const TABLE: &str = "city";

    /// Generate the SQL fragment '`table AS alias`'.
    ///
    /// # Arguments
    ///
    /// * `alias` is the table alias name.
    ///
    pub fn table_as(alias: &str) -> String {
        format!("{} AS {}", Self::TABLE, alias.to_string())
    }

    /// Generate the SQL fragment '`city AS ci ON r.id=ci.rid`'.
    ///
    /// # Arguments
    ///
    /// * `ci` is the [CityTbl] table alias name.
    /// * `r` is the [RegionTbl] table alias name.
    ///
    pub fn alias_join_region_as(ci: &str, r: &str) -> String {
        format!("{} ON {}={}", Self::table_as(ci), RegionTbl::Id.alias_column(r), CityTbl::Rid.alias_column(ci))
    }

    /// Get the SQL that will insert a row into the table.
    ///
    pub fn insert_sql() -> String {
        create_insert_sql(Self::TABLE, &Self::COLUMN_PARAM)
    }
}
impl TblSqlBuilder for CityTbl {
    /// The column names.
    ///
    fn column(&self) -> &'static str {
        match self {
            Self::Id => Self::COLUMN_PARAM[Self::Id as usize].0,
            Self::Rid => Self::COLUMN_PARAM[Self::Rid as usize].0,
            Self::Name => Self::COLUMN_PARAM[Self::Name as usize].0,
            Self::Latitude => Self::COLUMN_PARAM[Self::Latitude as usize].0,
            Self::Longitude => Self::COLUMN_PARAM[Self::Longitude as usize].0,
            Self::Tz => Self::COLUMN_PARAM[Self::Tz as usize].0,
        }
    }

    /// The column parameter names.
    /// 
    fn param(&self) -> &'static str {
        match self {
            Self::Id => Self::COLUMN_PARAM[Self::Id as usize].1,
            Self::Rid => Self::COLUMN_PARAM[Self::Rid as usize].1,
            Self::Name => Self::COLUMN_PARAM[Self::Name as usize].1,
            Self::Latitude => Self::COLUMN_PARAM[Self::Latitude as usize].1,
            Self::Longitude => Self::COLUMN_PARAM[Self::Longitude as usize].1,
            Self::Tz => Self::COLUMN_PARAM[Self::Tz as usize].1,
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
    use sql_query_builder as sql;

    #[test]
    fn row_insert() {
        // use an in-memory database
        let mut conn = db_connection(None).unwrap();
        cities::initialize_schema(&conn).unwrap();

        // set up the test environment
        let cid = cities::country::tests::insert_country(&mut conn, "", "");
        let rid = cities::region::tests::insert_region(&mut conn, "", "", cid);

        // add a test city
        let name = "City Name";
        let latitude = "-1";
        let longitude = "1";
        let tz = "utc";
        let insert_sql = CityTbl::insert_sql();
        let mut stmt = prepare_sql!(conn, &insert_sql, "failed to prepare city insert SQL").unwrap();
        let params = [
            named_param!(CityTbl::Rid, rid),
            named_param!(CityTbl::Name, name),
            named_param!(CityTbl::Latitude, latitude),
            named_param!(CityTbl::Longitude, longitude),
            named_param!(CityTbl::Tz, tz),
        ];
        execute_sql!(stmt, &params, "failed to insert test city").unwrap();

        // verify the row content
        let query_sql = sql::Select::new()
            .select(CityTbl::Rid.column())
            .select(CityTbl::Name.column())
            .select(CityTbl::Latitude.column())
            .select(CityTbl::Longitude.column())
            .select(CityTbl::Tz.column())
            .from(CityTbl::TABLE)
            .to_string();
        let mut stmt = prepare_cached_sql!(conn, &query_sql, "failed to prepare city verify SQL").unwrap();
        stmt.query_one([], |row| {
            assert_eq!(row.get::<_, i64>(CityTbl::Rid.column()).unwrap(), rid);
            assert_eq!(row.get::<_, String>(CityTbl::Name.column()).unwrap(), name);
            assert_eq!(row.get::<_, String>(CityTbl::Latitude.column()).unwrap(), latitude);
            assert_eq!(row.get::<_, String>(CityTbl::Longitude.column()).unwrap(), longitude);
            assert_eq!(row.get::<_, String>(CityTbl::Tz.column()).unwrap(), tz);
            Ok(())
        })
        .unwrap();
    }
}
