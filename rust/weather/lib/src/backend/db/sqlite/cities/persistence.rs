//! The [simple maps](https://simplemaps.com) city database CSV loader.
//!
use super::{CityMD, CountryMD, RegionMD};
use crate::backend::db::sqlite::{execute_sql, prepare_sql};
use rusqlite::{named_params, Connection, Error::QueryReturnedNoRows, Transaction};

/// Create a Simple Maps specific error message.
/// 
/// # Params
/// 
/// * `args` are passed to `format!` to create the error message.
/// 
macro_rules! err {
    ($($args:tt)*) => {
        Err(crate::Error(format!("Simple Maps loader {}", format!($($args)*))))
    };
}

// pull in what the inner module tests need
#[cfg(test)]
use crate::backend::{
    db::sqlite::{
        cities::{db_conn, init_schema, DB_FILENAME},
        commit_tx, create_tx,
    },
    testlib, WeatherDir
};

pub mod country {
    //! Manages the Cities country table.
    //!
    use super::*;

    /// The name of the database table.
    pub const TABLE: &str = "country";

    /// A specialized query that returns the database *ROWID* of a country.
    /// 
    /// # Arguments
    /// 
    /// 
    /// * `conn` is the database connection that will be used.
    /// * `country` provides the search criteria.
    pub fn get_id(conn: &Connection, country: &CountryMD) -> crate::Result<Option<i64>> {
        let query = format!("SELECT id FROM {TABLE} WHERE name = :name AND code = :code");
        let mut stmt = prepare_sql!(conn, &query, "failed to prepare {TABLE} ROWID query")?;
        match stmt.query_row(named_params! {":name": country.name, ":code": country.code}, |row| row.get(0)) {
            Err(QueryReturnedNoRows) => Ok(None),
            Ok(id) => Ok(Some(id)),
            Err(error) => err!("failed to get country {country} ROWID: {error}"),
        }
    }

    /// Add a country to the database table.
    /// 
    /// # Arguments
    /// 
    /// * `tx` is the database transaction that will be used.
    /// * `country` provides the metadata that will be added.
    /// 
    pub fn add(tx: &Transaction, country: &CountryMD) -> crate::Result<i64> {
        let name = country.name.trim();
        if name.is_empty() {
            err!("The country name cannot be empty")?;
        }
        let code = country.code.trim();
        if code.is_empty() {
            err!("The country code cannot be empty")?;
        }
        let sql = format!("INSERT INTO {TABLE} (name, code) VALUES (:name, :code)");
        let mut stmt = prepare_sql!(tx, &sql, "failed to prepare {TABLE} insert SQL")?;
        execute_sql!(stmt, named_params! {":name": name, ":code": code}, "failed to add country {country}")?;
        Ok(tx.last_insert_rowid())
    }

    /// Delete a country from the table.
    /// 
    /// # Arguments
    /// 
    /// * `tx` is the database transaction that will be used.
    /// * `country` provides the search criteria.
    /// 
    pub fn delete(tx: &Transaction, country: &CountryMD) -> crate::Result<()> {
        let sql = format!("DELETE FROM {TABLE} WHERE name=:name AND code=:code");
        let mut stmt = prepare_sql!(tx, &sql, "failed to prepare delete SQL")?;
        execute_sql!(
            stmt,
            named_params! {":name": country.name, ":code": country.code},
            "failed to delete country {country}"
        )?;
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn crud() {
            let fixture = testlib::TestFixture::create();
            let weather_dir = WeatherDir::from(&fixture);
            init_schema(&weather_dir).unwrap();

            let mut conn = db_conn(&weather_dir.file(DB_FILENAME)).unwrap();
            let country = CountryMD { name: "Name".to_string(), code: "Code".to_string() };

            // add the country
            let tx = create_tx!(conn, "failed to create insert tx").unwrap();
            let insert_id = add(&tx, &country).unwrap();
            commit_tx!(tx, "failed to commit insert tx").unwrap();

            // get the country id
            let sql = format!("select count(*) from {TABLE}");
            let count: i64 = conn.query_one(sql.as_str(), [], |row| row.get(0)).unwrap();
            assert_eq!(count, 1);
            assert_eq!(get_id(&conn, &country).unwrap(), Some(insert_id));

            // delete the country
            let tx = create_tx!(conn, "failed to create delete tx").unwrap();
            delete(&tx, &country).unwrap();
            commit_tx!(tx, "failed to commit delete tx").unwrap();
            let count: i64 = conn.query_one(sql.as_str(), [], |row| row.get(0)).unwrap();
            assert_eq!(count, 0);
        }
    }
}

pub mod region {
    //! Manages the Cities region table.
    //!
    use super::*;

    /// The name of the database table.
    pub const TABLE: &str = "region";

    /// A specialized query that returns the database *ROWID* of a region.
    /// 
    /// # Arguments
    /// 
    /// * `conn` is the database connection that will be used.
    /// * `coid` is the country *ROWID*.
    /// * `region` provides the search criteria.
    /// 
    #[cfg(test)]
    pub fn get_id(conn: &Connection, coid: i64, region: &RegionMD) -> crate::Result<Option<i64>> {
        let query = format!("SELECT id FROM {TABLE} WHERE coid=:coid AND name=:name AND code=:code");
        let mut stmt = prepare_sql!(conn, &query, "failed to prepare {TABLE} ROWID query")?;
        let params = named_params! {":coid": coid, ":name": region.name, ":code": region.code};
        match stmt.query_row(params, |row| row.get(0)) {
            Err(QueryReturnedNoRows) => Ok(None),
            Ok(id) => Ok(Some(id)),
            Err(error) => err!("failed to get region {region} ROWID: {error}"),
        }
    }

    /// Add a region to the database table.
    /// 
    /// # Arguments
    /// 
    /// * `tx` is the database transaction that will be used.
    /// * `coid` is the country *ROWID*.
    /// * `region` provides the metadata that will be added.
    /// 
    pub fn add(tx: &Transaction, coid: i64, region: &RegionMD) -> crate::Result<i64> {
        let name = region.name.trim();
        if name.is_empty() {
            err!("The region name cannot be empty")?;
        }
        let code = region.code.trim();
        if code.is_empty() {
            err!("The region code cannot be empty")?;
        }
        let sql = format!("INSERT INTO {TABLE} (coid, name, code) VALUES (:coid, :name, :code)");
        let mut stmt = prepare_sql!(tx, &sql, "failed to prepare {TABLE} insert SQL")?;
        let params = named_params! {":coid": coid, ":name": name, ":code": code};
        execute_sql!(stmt, params, "failed to add region {region}")?;
        Ok(tx.last_insert_rowid())
    }

    /// Delete the regions of a country from the table.
    /// 
    /// # Arguments
    /// 
    /// * `tx` is the database transaction that will be used.
    /// * `coid` is the country *ROWID*.
    /// 
    pub fn delete(tx: &Transaction, coid: i64) -> crate::Result<()> {
        let sql = format!("DELETE FROM {TABLE} WHERE coid=:coid");
        let mut stmt = prepare_sql!(tx, &sql, "failed to prepare {TABLE} delete SQL")?;
        execute_sql!(stmt, named_params! {":coid": coid}, "failed to delete regions")?;
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]

        fn crud() {
            let fixture = testlib::TestFixture::create();
            let weather_dir = WeatherDir::from(&fixture);
            init_schema(&weather_dir).unwrap();

            let mut conn = db_conn(&weather_dir.file(DB_FILENAME)).unwrap();
            let country = CountryMD { name: "Country".to_string(), code: "CON".to_string() };

            // add a country
            let tx = create_tx!(conn, "failed to create Country insert tx").unwrap();
            let coid = country::add(&tx, &country).unwrap();
            commit_tx!(tx, "failed to commit Country insert tx").unwrap();

            let region = RegionMD { name: "Region".to_string(), code: "REG".to_string() };

            // add a region
            let tx = create_tx!(conn, "failed to create Region insert tx").unwrap();
            let insert_id = add(&tx, coid, &region).unwrap();
            commit_tx!(tx, "failed to commit Region insert tx").unwrap();

            let sql = format!("select count(*) from {TABLE}");
            let count: i64 = conn.query_one(sql.as_str(), [], |row| row.get(0)).unwrap();
            assert_eq!(count, 1);
            assert_eq!(get_id(&conn, coid, &region).unwrap(), Some(insert_id));

            // delete the region
            let tx = create_tx!(conn, "failed to create delete tx").unwrap();
            delete(&tx, coid).unwrap();
            commit_tx!(tx, "failed to commit delete tx").unwrap();
            let count: i64 = conn.query_one(sql.as_str(), [], |row| row.get(0)).unwrap();
            assert_eq!(count, 0);
        }
    }
}

pub mod city {
    //! Manages the Cities City table
    //! 
    use super::*;

    /// The name of the database table.
    pub const TABLE: &str = "city";

    /// Add a city to the database table.
    /// 
    /// # Arguments
    /// 
    /// * `tx` is the database transaction that will be used.
    /// * `rid` is the region *ROWID*.
    /// * `city` provides the metadata that will be added.
    /// 
    pub fn insert(tx: &Transaction, rid: i64, city: &CityMD) -> crate::Result<i64> {
        macro_rules! err_if_empty {
            ($value: expr, $descr: expr) => {{
                let v = $value.trim();
                match v.is_empty() {
                    true => err!("The city {} cannot be empty", $descr),
                    false => Ok(v),
                }
            }};
        }
        let name = err_if_empty!(city.name, "name")?;
        let lat = err_if_empty!(city.latitude, "latitude")?;
        let lng = err_if_empty!(city.longitude, "longitude")?;
        let tz = err_if_empty!(city.timezone, "timezone")?;
        let sql = format!("INSERT INTO {TABLE} (rid, name, lat, lng, tz) VALUES (:rid, :name, :lat, :lng, :tz)");
        let mut stmt = prepare_sql!(tx, &sql, "failed to prepare {TABLE} insert SQL")?;
        let params = named_params! {":rid": rid, ":name": name, ":lat": lat, ":lng": lng, ":tz": tz};
        execute_sql!(stmt, params, "failed to add city {city}")?;
        Ok(tx.last_insert_rowid())
    }

    /// Delete the cities of a country from the table.
    /// 
    /// # Arguments
    /// 
    /// * `tx` is the database transaction that will be used.
    /// * `coid` is the country *ROWID*.
    /// 
    pub fn delete(tx: &Transaction, coid: i64) -> crate::Result<()> {
        let sql = format!(
            r#"
            DELETE FROM {TABLE} WHERE id IN (
                SELECT c.id FROM {TABLE} AS c
                INNER JOIN {} AS r ON r.id = c.rid
                WHERE r.coid = :coid
            )
        "#,
            region::TABLE
        );
        let mut stmt = prepare_sql!(tx, &sql, "failed to prepare {TABLE} delete SQL")?;
        execute_sql!(stmt, named_params! {":coid": coid}, "failed to delete cities")?;
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn crud() {
            let fixture = testlib::TestFixture::create();
            let weather_dir = WeatherDir::from(&fixture);
            init_schema(&weather_dir).unwrap();

            // set up the tests
            let mut conn = db_conn(&weather_dir.file(DB_FILENAME)).unwrap();
            let country = CountryMD { name: "Name".to_string(), code: "Code".to_string() };
            let region = RegionMD { name: "Region".to_string(), code: "REG".to_string() };
            let tx = create_tx!(conn, "failed to create setup insert tx").unwrap();
            let coid = country::add(&tx, &country).unwrap();
            let rid = region::add(&tx, coid, &region).unwrap();
            commit_tx!(tx, "failed to commit setup insert tx").unwrap();

            macro_rules! city {
                ($name: expr) => {
                    CityMD {
                        name: $name.to_string(),
                        region: region.clone(),
                        latitude: "12.345".to_string(),
                        longitude: "54.321".to_string(),
                        timezone: "UTC".to_string(),
                    }
                };
            }
            // add a city
            let tx = create_tx!(conn, "failed to create {TABLE} insert tx").unwrap();
            insert(&tx, rid, &city!("City 1")).unwrap();
            insert(&tx, rid, &city!("City 2")).unwrap();
            insert(&tx, rid, &city!("City 3")).unwrap();
            commit_tx!(tx, "failed to commit {TABLE} insert tx").unwrap();
            let sql = format!("SELECT count(*) from {TABLE}");
            let count: i64 = conn.query_one(sql.as_str(), [], |row| row.get(0)).unwrap();
            assert_eq!(count, 3);

            // delete cities
            let tx = create_tx!(conn, "failed to create {TABLE} delete tx").unwrap();
            delete(&tx, rid).unwrap();
            commit_tx!(tx, "failed to commit {TABLE} delete tx").unwrap();
            let count: i64 = conn.query_one(sql.as_str(), [], |row| row.get(0)).unwrap();
            assert_eq!(count, 0);
        }
    }
}
