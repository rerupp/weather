//! The cities database persistence module.
//!
use super::{CityMD, CountryMD, RegionMD};
use crate::backend::db::sqlite::{execute_sql, prepare_cached_sql, prepare_sql};
use rusqlite::{Connection, Error::QueryReturnedNoRows, Transaction};
use sql_query_builder as sql;

#[doc(hidden)]
/// Create a cities persistence specific error message.
///
/// # Params
///
/// * `args` are passed to `format!` to create the error message.
///
macro_rules! err {
    ($($args:tt)*) => {
        Err(crate::Error(format!("Cities persistence {}", format!($($args)*))))
    };
}

// pull in what the inner module tests need
#[cfg(test)]
use crate::backend::{
    db::sqlite::{cities::db_conn, commit_tx, create_tx},
    testlib, WeatherDir,
};

pub mod country {
    //! Manages the Cities country table.
    //!
    use super::*;
    use crate::backend::db::sqlite::tables::{cities::CountryTbl, TblSqlBuilder};

    /// A specialized query that returns the database *ROWID* of a country.
    ///
    /// # Arguments
    ///
    /// * `conn` is the database connection that will be used.
    /// * `country` provides the search criteria.
    ///
    pub fn get_id(conn: &Connection, country: &CountryMD) -> crate::Result<Option<i64>> {
        let get_sql = sql::Select::new()
            .select(CountryTbl::Id.column())
            .from(CountryTbl::TABLE)
            .where_and(&CountryTbl::Name.where_param())
            .where_and(&CountryTbl::Code.where_param())
            .to_string();
        let mut stmt = prepare_sql!(conn, &get_sql, "failed to prepare {} ROWID query", CountryTbl::TABLE)?;
        let params = [(CountryTbl::Name.param(), &country.name), (CountryTbl::Code.param(), &country.code)];
        match stmt.query_row(&params, |row| row.get(0)) {
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
    pub fn insert(tx: &Transaction, country: &CountryMD) -> crate::Result<i64> {
        // probably should also check if the country exists
        let name = country.name.trim();
        if name.is_empty() {
            err!("The country name cannot be empty")?;
        }
        let code = country.code.trim();
        if code.is_empty() {
            err!("The country code cannot be empty")?;
        }

        // insert the country
        let insert_sql = CountryTbl::insert_sql();
        let mut stmt = prepare_cached_sql!(tx, &insert_sql, "failed to prepare {} insert SQL", CountryTbl::TABLE)?;
        let params = [(CountryTbl::Name.param(), name), (CountryTbl::Code.param(), code)];
        execute_sql!(stmt, &params, "failed to add country {country}")?;
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
        let delete_sql = sql::Delete::new()
            .delete_from(CountryTbl::TABLE)
            .where_and(&CountryTbl::Name.where_param())
            .where_and(&CountryTbl::Code.where_param())
            .to_string();
        let mut stmt = prepare_sql!(tx, &delete_sql, "failed to prepare delete SQL")?;
        let params = [(CountryTbl::Name.param(), &country.name), (CountryTbl::Code.param(), &country.code)];
        execute_sql!(stmt, &params, "failed to delete country {country}")?;
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::backend::db::sqlite::tables;

        #[test]
        fn crud() {
            let fixture = testlib::TestFixture::create();
            let weather_dir = WeatherDir::from(&fixture);
            let mut conn = db_conn!(&weather_dir).unwrap();

            tables::cities::initialize_schema(&conn).unwrap();

            let country = CountryMD { name: "Name".to_string(), code: "Code".to_string() };

            // add the country
            let tx = create_tx!(conn, "failed to create insert tx").unwrap();
            let insert_id = insert(&tx, &country).unwrap();
            commit_tx!(tx, "failed to commit insert tx").unwrap();

            // get the country id
            let sql = format!("select count(*) from {}", CountryTbl::TABLE);
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
    use crate::backend::db::sqlite::tables::{named_param, cities::RegionTbl, TblSqlBuilder};

    /// A specialized query that returns the database *ROWID* of a region.
    ///
    /// # Arguments
    ///
    /// * `conn` is the database connection that will be used.
    /// * `cid` is the country *ROWID*.
    /// * `region` provides the search criteria.
    ///
    #[cfg(test)]
    pub fn get_id(conn: &Connection, cid: i64, region: &RegionMD) -> crate::Result<Option<i64>> {
        let get_sql = sql::Select::new()
            .select(RegionTbl::Id.column())
            .from(RegionTbl::TABLE)
            .where_and(&RegionTbl::Cid.where_param())
            .where_and(&RegionTbl::Name.where_param())
            .where_and(&RegionTbl::Code.where_param())
            .to_string();
        let mut stmt = prepare_sql!(conn, &get_sql, "failed to prepare {} ROWID query", RegionTbl::TABLE)?;
        let params = [
            named_param!(RegionTbl::Cid, cid),
            named_param!(RegionTbl::Name, &region.name),
            named_param!(RegionTbl::Code, &region.code),
        ];
        match stmt.query_row(&params, |row| row.get(0)) {
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
    /// * `cid` is the country *ROWID*.
    /// * `region` provides the metadata that will be added.
    ///
    pub fn insert(tx: &Transaction, cid: i64, region: &RegionMD) -> crate::Result<i64> {
        let name = region.name.trim();
        if name.is_empty() {
            err!("The region name cannot be empty")?;
        }
        let code = region.code.trim();
        if code.is_empty() {
            err!("The region code cannot be empty")?;
        }
        let insert_sql = RegionTbl::insert_sql();
        let mut stmt = prepare_cached_sql!(tx, &insert_sql, "failed to prepare {} insert SQL", RegionTbl::TABLE)?;
        let params = [
            named_param!(RegionTbl::Cid, cid),
            named_param!(RegionTbl::Name, &region.name),
            named_param!(RegionTbl::Code, &region.code),
        ];
        execute_sql!(stmt, &params, "failed to add region {region}")?;
        Ok(tx.last_insert_rowid())
    }

    /// Delete the regions of a country from the table.
    ///
    /// # Arguments
    ///
    /// * `tx` is the database transaction that will be used.
    /// * `cid` is the country *ROWID*.
    ///
    pub fn delete(tx: &Transaction, cid: i64) -> crate::Result<()> {
        let delete_sql =
            sql::Delete::new().delete_from(RegionTbl::TABLE).where_clause(&RegionTbl::Cid.where_param()).to_string();
        let mut stmt = prepare_sql!(tx, &delete_sql, "failed to prepare {} delete SQL", RegionTbl::TABLE)?;
        let params = [(RegionTbl::Cid.param(), &cid)];
        execute_sql!(stmt, &params, "failed to delete regions")?;
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::backend::db::sqlite::tables;
        #[test]

        fn crud() {
            let fixture = testlib::TestFixture::create();
            let weather_dir = WeatherDir::from(&fixture);
            let mut conn = db_conn!(&weather_dir).unwrap();

            tables::cities::initialize_schema(&conn).unwrap();

            let country = CountryMD { name: "Country".to_string(), code: "CON".to_string() };

            // add a country
            let tx = create_tx!(conn, "failed to create Country insert tx").unwrap();
            let coid = country::insert(&tx, &country).unwrap();
            commit_tx!(tx, "failed to commit Country insert tx").unwrap();

            let region = RegionMD { name: "Region".to_string(), code: "REG".to_string() };

            // add a region
            let tx = create_tx!(conn, "failed to create Region insert tx").unwrap();
            let insert_id = insert(&tx, coid, &region).unwrap();
            commit_tx!(tx, "failed to commit Region insert tx").unwrap();

            let sql = format!("select count(*) from {}", RegionTbl::TABLE);
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
    use crate::backend::db::sqlite::{
        prepare_cached_sql,
        tables::{named_param, cities::{CityTbl, RegionTbl}, TblSqlBuilder},
    };

    /// Add a city to the database table.
    ///
    /// # Arguments
    ///
    /// * `tx` is the database transaction that will be used.
    /// * `rid` is the region *ROWID*.
    /// * `city` provides the metadata that will be added.
    ///
    pub fn insert(tx: &Transaction, rid: i64, city: &CityMD) -> crate::Result<i64> {
        // do some quick validation
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

        // insert the city
        let insert_sql = CityTbl::insert_sql();
        let mut stmt = prepare_cached_sql!(tx, &insert_sql, "failed to prepare {} insert SQL", CityTbl::TABLE)?;
        let params = [
            named_param!(CityTbl::Rid, rid),
            named_param!(CityTbl::Name, name),
            named_param!(CityTbl::Latitude, lat),
            named_param!(CityTbl::Longitude, lng),
            named_param!(CityTbl::Tz, tz),
        ];
        execute_sql!(stmt, &params, "failed to add city {city}")?;
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
        let r = "r";
        let c = "c";
        let inner_select_sql = sql::Select::new()
            .select(&CityTbl::Id.alias_column(c))
            .from(&RegionTbl::table_as(r))
            .inner_join(&CityTbl::alias_join_region_as(c, r))
            .where_clause(&RegionTbl::Cid.alias_where_param(r))
            .to_string();
        let delete_sql = sql::Delete::new()
            .delete_from(CityTbl::TABLE)
            .where_clause(&format!("{} IN ({})", CityTbl::Id.column(), inner_select_sql))
            .to_string();
        let mut stmt = prepare_sql!(tx, &delete_sql, "failed to prepare {} delete SQL", CityTbl::TABLE)?;
        let params = [(RegionTbl::Cid.param(), &coid)];
        execute_sql!(stmt, &params, "failed to delete cities")?;
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::backend::db::sqlite::tables;
        #[test]
        fn crud() {
            let fixture = testlib::TestFixture::create();
            let weather_dir = WeatherDir::from(&fixture);
            let mut conn = db_conn!(&weather_dir).unwrap();
            tables::cities::initialize_schema(&conn).unwrap();

            // set up the tests
            let country = CountryMD { name: "Name".to_string(), code: "Code".to_string() };
            let region = RegionMD { name: "Region".to_string(), code: "REG".to_string() };
            let tx = create_tx!(conn, "failed to create setup insert tx").unwrap();
            let coid = country::insert(&tx, &country).unwrap();
            let rid = region::insert(&tx, coid, &region).unwrap();
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
            let tx = create_tx!(conn, "failed to create {} insert tx", CityTbl::TABLE).unwrap();
            insert(&tx, rid, &city!("City 1")).unwrap();
            insert(&tx, rid, &city!("City 2")).unwrap();
            insert(&tx, rid, &city!("City 3")).unwrap();
            commit_tx!(tx, "failed to commit {} insert tx", CityTbl::TABLE).unwrap();
            let sql = format!("SELECT count(*) from {}", CityTbl::TABLE);
            let count: i64 = conn.query_one(sql.as_str(), [], |row| row.get(0)).unwrap();
            assert_eq!(count, 3);

            // delete cities
            let tx = create_tx!(conn, "failed to create {} delete tx", CityTbl::TABLE).unwrap();
            delete(&tx, rid).unwrap();
            commit_tx!(tx, "failed to commit {} delete tx", CityTbl::TABLE).unwrap();
            let count: i64 = conn.query_one(sql.as_str(), [], |row| row.get(0)).unwrap();
            assert_eq!(count, 0);
        }
    }
}
