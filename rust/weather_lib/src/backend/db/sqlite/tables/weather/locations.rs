//! The locations table definition.
//!
use super::{create_insert_sql, TblSqlBuilder};

pub const SCHEMA: &'static str = r"
-- The weather locations table
CREATE TABLE IF NOT EXISTS locations
(
    id           INTEGER PRIMARY KEY,
    country_name TEXT NOT NULL COLLATE NOCASE,
    country_code TEXT NOT NULL COLLATE NOCASE,
    region_name  TEXT NOT NULL COLLATE NOCASE,
    region_code  TEXT NOT NULL COLLATE NOCASE,
    city_name    TEXT NOT NULL COLLATE NOCASE,
    alias        TEXT NOT NULL COLLATE NOCASE,
    latitude     TEXT NOT NULL,
    longitude    TEXT NOT NULL,
    tz           TEXT NOT NULL COLLATE NOCASE
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_locations_alias ON locations (alias COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS idx_locations_city_name ON locations (city_name COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS idx_locations_country_name ON locations (country_name COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS idx_locations_country_code ON locations (country_code COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS idx_locations_region_name ON locations (region_name COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS idx_locations_region_code ON locations (region_code COLLATE NOCASE);
";

/// The Location table columns and name parameters.
///
#[derive(Debug)]
pub enum LocationsTbl {
    // Variants MUST follow the column order
    Id,
    CountryName,
    CountryCode,
    RegionName,
    RegionCode,
    CityName,
    Alias,
    Latitude,
    Longitude,
    Tz,
}

impl LocationsTbl {
    /// The row column name and default named parameter pairs.
    ///
    const COLUMN_PARAM: [(&str, &str); 10] = [
        // the array MUST be in enum order to stay in sync
        ("id", ":id"),
        ("country_name", ":country_name"),
        ("country_code", ":country_code"),
        ("region_name", ":region_name"),
        ("region_code", ":region_code"),
        ("city_name", ":city_name"),
        ("alias", ":alias"),
        ("latitude", ":latitude"),
        ("longitude", ":longitude"),
        ("tz", ":tz"),
    ];

    /// The schema table name.
    ///
    pub const TABLE: &str = "locations";

    /// Generate the SQL fragment '`table AS alias`'.
    ///
    /// # Arguments
    ///
    /// * `alias` is the table alias name.
    ///
    pub fn table_as(alias: &str) -> String {
        format!("{} AS {}", Self::TABLE, alias)
    }

    /// Generate the SQL that will insert a row into the table.
    ///
    pub fn insert_sql() -> String {
        create_insert_sql(Self::TABLE, &Self::COLUMN_PARAM)
    }
}
impl TblSqlBuilder for LocationsTbl {
    /// The column names.
    ///
    fn column(&self) -> &'static str {
        match self {
            Self::Id => Self::COLUMN_PARAM[Self::Id as usize].0,
            Self::CountryName => Self::COLUMN_PARAM[Self::CountryName as usize].0,
            Self::CountryCode => Self::COLUMN_PARAM[Self::CountryCode as usize].0,
            Self::RegionName => Self::COLUMN_PARAM[Self::RegionName as usize].0,
            Self::RegionCode => Self::COLUMN_PARAM[Self::RegionCode as usize].0,
            Self::CityName => Self::COLUMN_PARAM[Self::CityName as usize].0,
            Self::Alias => Self::COLUMN_PARAM[Self::Alias as usize].0,
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
            Self::CountryName => Self::COLUMN_PARAM[Self::CountryName as usize].1,
            Self::CountryCode => Self::COLUMN_PARAM[Self::CountryCode as usize].1,
            Self::RegionName => Self::COLUMN_PARAM[Self::RegionName as usize].1,
            Self::RegionCode => Self::COLUMN_PARAM[Self::RegionCode as usize].1,
            Self::CityName => Self::COLUMN_PARAM[Self::CityName as usize].1,
            Self::Alias => Self::COLUMN_PARAM[Self::Alias as usize].1,
            Self::Latitude => Self::COLUMN_PARAM[Self::Latitude as usize].1,
            Self::Longitude => Self::COLUMN_PARAM[Self::Longitude as usize].1,
            Self::Tz => Self::COLUMN_PARAM[Self::Tz as usize].1,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{
        backend::db::sqlite::{
            db_connection, err, execute_sql, prepare_cached_sql, prepare_sql,
            tables::{named_param, weather::initialize_schema},
        },
        entities::Location,
    };
    use rusqlite::Connection;
    use sql_query_builder as sql;

    // a helper for testcases that will need to insert a location
    pub fn insert_location(conn: &mut Connection, location: Location) -> i64 {
        let insert_sql = LocationsTbl::insert_sql();
        let mut stmt = prepare_sql!(conn, &insert_sql, "failed to prepare insert SQL").unwrap();
        let params = [
            named_param!(LocationsTbl::CountryName, location.country_name),
            named_param!(LocationsTbl::CountryCode, location.country_code),
            named_param!(LocationsTbl::RegionName, location.region_name),
            named_param!(LocationsTbl::RegionCode, location.region_code),
            named_param!(LocationsTbl::CityName, location.city_name),
            named_param!(LocationsTbl::Alias, location.alias),
            named_param!(LocationsTbl::Latitude, location.latitude),
            named_param!(LocationsTbl::Longitude, location.longitude),
            named_param!(LocationsTbl::Tz, location.tz),
        ];
        execute_sql!(stmt, &params, "failed to insert locations").unwrap();
        conn.last_insert_rowid()
    }

    #[test]
    fn row_insert() {
        // use an in-memory database
        let mut conn = db_connection(None).unwrap();
        initialize_schema(&conn).unwrap();

        // add a location row
        let location = Location {
            country_name: "country name".to_string(),
            country_code: "country code".to_string(),
            region_name: "region name".to_string(),
            region_code: "region code".to_string(),
            city_name: "city name".to_string(),
            alias: "alias name".to_string(),
            latitude: "1".to_string(),
            longitude: "-1".to_string(),
            tz: "utc".to_string(),
        };
        insert_location(&mut conn, location.clone());

        // verify the row content
        let query_sql = sql::Select::new()
            .select(LocationsTbl::CountryName.column())
            .select(LocationsTbl::CountryCode.column())
            .select(LocationsTbl::RegionName.column())
            .select(LocationsTbl::RegionCode.column())
            .select(LocationsTbl::CityName.column())
            .select(LocationsTbl::Alias.column())
            .select(LocationsTbl::Latitude.column())
            .select(LocationsTbl::Longitude.column())
            .select(LocationsTbl::Tz.column())
            .from(LocationsTbl::TABLE)
            .to_string();
        let mut stmt = prepare_cached_sql!(conn, &query_sql, "failed to prepare verify query SQL").unwrap();
        stmt.query_one([], |row| {
            assert_eq!(row.get::<_, String>(LocationsTbl::CountryName.column()).unwrap(), location.country_name);
            assert_eq!(row.get::<_, String>(LocationsTbl::CountryCode.column()).unwrap(), location.country_code);
            assert_eq!(row.get::<_, String>(LocationsTbl::RegionName.column()).unwrap(), location.region_name);
            assert_eq!(row.get::<_, String>(LocationsTbl::RegionCode.column()).unwrap(), location.region_code);
            assert_eq!(row.get::<_, String>(LocationsTbl::CityName.column()).unwrap(), location.city_name);
            assert_eq!(row.get::<_, String>(LocationsTbl::Alias.column()).unwrap(), location.alias);
            assert_eq!(row.get::<_, String>(LocationsTbl::Latitude.column()).unwrap(), location.latitude);
            assert_eq!(row.get::<_, String>(LocationsTbl::Longitude.column()).unwrap(), location.longitude);
            assert_eq!(row.get::<_, String>(LocationsTbl::Tz.column()).unwrap(), location.tz);
            Ok(())
        })
        .unwrap();
    }
}
