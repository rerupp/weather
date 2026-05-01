//! The SQLite Cities database implementation.
//!
mod persistence;
use persistence::{city, country, region};
mod query;
mod simple_maps;

use super::{commit_tx, create_tx, db_connection, prepare_sql, query_rows, SqlResult};
use crate::{
    admin_prelude::{CitiesDetails, CountryDetails, RegionDetails},
    backend::filesys::{WeatherDir, WeatherFile},
    entities::LocationFilter,
    prelude::City,
};
use rusqlite::{Connection, Row};
use std::{collections::HashMap, path::PathBuf};

/// The name of the Cities database;
const DB_FILENAME: &'static str = "cities.db";

/// Create an error from the locations specific error message.
///
/// # Params
///
/// * `args` will be passed to `format!` to create the error message.
///
macro_rules! err {
    ($($args:tt)*) => {
        Err(crate::Error::from(format!("Cities {}", format!($($args)*))))
    };
}

/// Create a database connection.
///
/// # Arguments
///
/// * `db_file` is the database file that will be opened.
///
#[inline]
fn db_conn(db_file: &WeatherFile) -> crate::Result<Connection> {
    match db_connection(Some(db_file)) {
        Ok(conn) => Ok(conn),
        Err(error) => err!("database could not be opened: {error:?}"),
    }
}

/// Create a database connection if the database has already been initialized.
///
/// # Arguments
///
/// * `db_file` is the database file that will be opened.
///
fn db_conn_opt(db_file: &WeatherFile) -> crate::Result<Option<Connection>> {
    match db_file.exists() {
        false => Ok(None),
        true => {
            let conn = db_conn(db_file)?;
            // unless things are AFU checking if one of the tables exist should be enough
            let query = "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='country')";
            match conn.query_one(query, [], |row| row.get::<_, i64>(0)) {
                Err(error) => err!("failed to read schema: {error:?}"),
                Ok(exists) => match exists > 0 {
                    true => Ok(Some(conn)),
                    false => Ok(None),
                },
            }
        }
    }
}

/// Checks the database to see if it has been initialized.
///
/// # Arguments
///
/// * `weather_dir` is the weather history data directory.
///
pub fn is_initialized(weather_dir: &WeatherDir) -> bool {
    match db_conn_opt(&weather_dir.file(DB_FILENAME)) {
        Err(error) => {
            log::error!("{error:?}");
            false
        }
        Ok(None) => false,
        Ok(Some(_)) => true,
    }
}

/// Initialize the database schema.
///
/// # Arguments
///
/// * `weather_dir` is the weather history data directory.
///
pub fn init_schema(weather_dir: &WeatherDir) -> crate::Result<()> {
    let schema_sql = include_str!("cities/schema.sql");
    // let db_filename = weather_dir.file(DB_FILENAME);
    let conn = db_conn(&weather_dir.file(DB_FILENAME))?;
    if let Err(error) = conn.execute_batch(schema_sql) {
        err!("failed to initialize the database schema: {:?}", error)?;
    }
    Ok(())
}

/// Delete the Cities database
///
/// # Arguments
///
/// * `weather_dir` is the weather history data directory.
///
pub fn delete(weather_dir: &WeatherDir) -> crate::Result<()> {
    let db = weather_dir.file(DB_FILENAME);
    if db.exists() {
        if let Err(error) = db.remove() {
            err!("database was not deleted: {error:?}")?;
        }
    }
    Ok(())
}

/// Load a Simple Maps country database file.
///
/// # Arguments
///
/// * `weather_dir` is the weather history data directory.
/// * `csv_filename` identifies the CSV database.
/// * `reload` forces any previous country entries to be deleted prior to loading the new data.
///
pub fn load(weather_dir: &WeatherDir, csv_filename: &str, reload: bool) -> crate::Result<usize> {
    // verify the database is ready
    let mut conn = match db_conn_opt(&weather_dir.file(DB_FILENAME))? {
        None => err!("database does not appear to be initialized."),
        Some(conn) => Ok(conn),
    }?;

    // parse the csv file
    let (country, regions, cities) = simple_maps::parse(&PathBuf::from(csv_filename))?;

    // set up the load
    let tx = create_tx!(conn, "could not create a load tx")?;
    if reload {
        if let Some(coid) = country::get_id(&tx, &country)? {
            city::delete(&tx, coid)?;
            region::delete(&tx, coid)?;
            country::delete(&tx, &country)?;
        }
    }

    // add the country
    let coid = country::add(&tx, &country)?;

    // add the regions
    let mut region_to_id = HashMap::new();
    for region in regions {
        let rid = region::add(&tx, coid, &region)?;
        region_to_id.insert(region, rid);
    }

    // add the cities
    for city in &cities {
        let rid = region_to_id[&city.region];
        city::insert(&tx, rid, city)?;
    }

    commit_tx!(tx, "failed to commit load")?;
    Ok(cities.len())
}

/// Get collection of cities from the database.
///
/// # Arguments
///
/// * `weather_dir` is the weather history data directory.
/// * `filters` is used to find cities.
/// * `limit` restricts the number of cities returned.
///
pub fn get(weather_dir: &WeatherDir, filters: Option<Vec<LocationFilter>>, limit: usize) -> crate::Result<Vec<City>> {
    // get the database connection
    let conn = match db_conn_opt(&weather_dir.file(DB_FILENAME))? {
        None => err!("database does not appear to be initialized"),
        Some(conn) => Ok(conn),
    }?;

    // execute the query
    let select = query::city(filters, limit)?;
    let mut stmt = prepare_sql!(conn, &select, "failed to prepare select SQL")?;
    let mut rows = query_rows!(stmt, [], "failed to query city data")?;
    let mut cities = Vec::new();
    loop {
        match rows.next() {
            Err(error) => err!("failed to get next city row: {:?}", error)?,
            Ok(None) => break,
            Ok(Some(row)) => {
                // this function captures the potential rusqlite errors that can occur
                #[inline]
                fn row_to_city(row: &Row) -> SqlResult<City> {
                    Ok(City {
                        country_name: row.get(query::COUNTRY_NAME)?,
                        country_code: row.get(query::COUNTRY_CODE)?,
                        region_name: row.get(query::REGION_NAME)?,
                        region_code: row.get(query::REGION_CODE)?,
                        name: row.get(query::CITY_NAME)?,
                        latitude: row.get(query::LATITUDE)?,
                        longitude: row.get(query::LONGITUDE)?,
                        tz: row.get(query::TIMEZONE)?,
                    })
                }
                match row_to_city(row) {
                    Ok(city) => cities.push(city),
                    Err(error) => err!("failed to create city from row: {:?}", error)?,
                }
            }
        }
    }
    Ok(cities)
}

/// Get details about the Cities database.
///
/// # Arguments
///
/// * `weather_dir` is the weather history data directory.
///
pub fn details(weather_dir: &WeatherDir) -> crate::Result<Option<CitiesDetails>> {
    let db_file = weather_dir.file(DB_FILENAME);
    // get the database connection
    let conn_opt = db_conn_opt(&weather_dir.file(DB_FILENAME))?;
    if conn_opt.is_none() {
        return Ok(None);
    }
    let conn = conn_opt.unwrap();

    // execute the query
    let select = query::details();
    let mut stmt = prepare_sql!(conn, &select, "failed to prepare details SQL")?;
    let mut rows = query_rows!(stmt, [], "failed to query city details")?;

    // collect the query details
    let mut countries: HashMap<CountryMD, Vec<RegionDetails>> = HashMap::new();
    loop {
        match rows.next() {
            Err(error) => err!("failed to get next city details row: {:?}", error)?,
            Ok(None) => break,
            Ok(Some(row)) => {
                // capture the potential rusqlite Errors in this function
                #[inline]
                fn to_country_region_details(row: &Row) -> SqlResult<(CountryMD, RegionDetails)> {
                    let country =
                        CountryMD { name: row.get(query::COUNTRY_NAME)?, code: row.get(query::COUNTRY_CODE)? };
                    let region_details = RegionDetails {
                        name: row.get(query::REGION_NAME)?,
                        code: row.get(query::REGION_CODE)?,
                        city_count: row.get::<_, i64>(query::CITY_COUNT)? as usize,
                    };
                    Ok((country, region_details))
                }
                match to_country_region_details(&row) {
                    Err(error) => err!("failed to create country details: {:?}", error)?,
                    Ok((country_md, region_details)) => {
                        if let Some(country) = countries.get_mut(&country_md) {
                            country.push(region_details);
                        } else {
                            countries.insert(country_md, vec![region_details]);
                        }
                    }
                }
            }
        }
    }

    // create the details
    let mut cities_details = CitiesDetails {
        db_size: db_file.size() as usize,
        country_details: countries
            .drain()
            .map(|(country_md, region_details)| CountryDetails {
                name: country_md.name,
                code: country_md.code,
                region_details,
            })
            .collect(),
    };
    // sort the country details
    cities_details.country_details.sort_unstable_by(|lhs, rhs| lhs.name.cmp(&rhs.name));
    Ok(Some(cities_details))
}

/// The country related metadata mined from the CSV database.
///
#[derive(Debug, Hash, PartialEq, Eq, Clone, Ord, PartialOrd)]
struct CountryMD {
    /// The country name such as *United States* or *Canada*.
    name: String,
    /// The country code such as *US* or *CA*.
    code: String,
}
impl std::fmt::Display for CountryMD {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.code)
    }
}

/// The region related metadata mined from the CSV database.
///
#[derive(Debug, Hash, PartialEq, Eq, Clone, Ord, PartialOrd)]
struct RegionMD {
    /// The region name such as *Arizona* or *Ontario*.
    name: String,
    /// The region code such as *AZ* or *ON*.
    code: String,
}
impl std::fmt::Display for RegionMD {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.code)
    }
}

/// The City metadata that is mined from the CSV database.
///
#[derive(Debug)]
struct CityMD {
    /// The name of the city.
    name: String,
    // the cities region metadata
    region: RegionMD,
    /// The city latitude.
    latitude: String,
    /// The city longitude.
    longitude: String,
    /// The city timezone.
    timezone: String,
}
impl std::fmt::Display for CityMD {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.region.name)
    }
}
