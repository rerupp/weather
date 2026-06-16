//! The SQLite Cities database implementation.
//!
mod persistence;
use persistence::{city, country, region};

mod simple_maps;

use crate::{
    admin_prelude::{CitiesDetails, CountryDetails, RegionDetails},
    backend::{
        db::sqlite::{
            commit_tx, create_tx, generate_sql_match_condition, prepare_sql, query_rows,
            tables::{self, cities::{CityTbl, CountryTbl, RegionTbl}, TblSqlBuilder},
        },
        filesys::WeatherDir,
    },
    entities::LocationFilter,
    prelude::City,
};
use rusqlite::Connection;
use sql_query_builder as sql;
use std::{collections::HashMap, path::PathBuf};

/// The name of the Cities database;
pub const CITIES_DB_FILENAME: &'static str = "cities.db";

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

macro_rules! db_conn {
    ($weather_dir: expr) => {
        crate::backend::db::sqlite::db_connection(Some(
            &$weather_dir.file(crate::backend::db::sqlite::cities::CITIES_DB_FILENAME),
        ))
    };
}
pub(super) use db_conn;

fn db_conn_if_ready(weather_dir: &WeatherDir) -> crate::Result<Connection> {
    match weather_dir.file(CITIES_DB_FILENAME).exists() {
        false => err!("database is not available."),
        true => {
            let conn = db_conn!(weather_dir)?;
            match tables::cities::is_schema_initialized(&conn)? {
                false => err!("database schema has not been initialized."),
                true => Ok(conn),
            }
        }
    }
}

/// Delete the Cities database
///
/// # Arguments
///
/// * `weather_dir` is the weather history data directory.
///
pub fn delete(weather_dir: &WeatherDir) -> crate::Result<()> {
    let db = weather_dir.file(CITIES_DB_FILENAME);
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
    let mut conn = db_conn_if_ready(weather_dir)?;
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
    let coid = country::insert(&tx, &country)?;

    // add the regions
    let mut region_to_id = HashMap::new();
    for region in regions {
        let rid = region::insert(&tx, coid, &region)?;
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
/// * `filters_opt` is used to find cities.
/// * `limit` restricts the number of cities returned.
///
pub fn get(
    weather_dir: &WeatherDir,
    filters_opt: Option<Vec<LocationFilter>>,
    limit: usize,
) -> crate::Result<Vec<City>> {
    // get the database connection
    let conn = db_conn_if_ready(weather_dir)?;

    // build the query
    let co = "co";
    let r = "r";
    let ci = "ci";
    let country_name = "country_name";
    let country_code = "country_code";
    let region_name = "region_name";
    let region_code = "region_code";
    let city_name = "city_name";
    let mut query = sql::Select::new()
        .select(&CountryTbl::Name.alias_column_as_name(co, country_name))
        .select(&CountryTbl::Code.alias_column_as_name(co, country_code))
        .select(&RegionTbl::Name.alias_column_as_name(r, region_name))
        .select(&RegionTbl::Code.alias_column_as_name(r, region_code))
        .select(&CityTbl::Name.alias_column_as_name(ci, city_name))
        .select(&CityTbl::Latitude.alias_column_as_column(ci))
        .select(&CityTbl::Longitude.alias_column_as_column(ci))
        .select(&CityTbl::Tz.alias_column_as_column(ci))
        .from(&CountryTbl::table_as(co))
        .inner_join(&RegionTbl::alias_join_country_as(r, co))
        .inner_join(&CityTbl::alias_join_region_as(ci, r));
    if let Some(filters) = filters_opt {
        for filter in filters {
            let mut filter_sql = vec![];
            if let Some(city_filter) = &filter.city {
                let name_match = generate_sql_match_condition(city_name, city_filter)?;
                filter_sql.push(name_match);
            }
            if let Some(region_filter) = &filter.region {
                let region_name_match = generate_sql_match_condition(region_name, region_filter)?;
                let region_code_match = generate_sql_match_condition(region_code, region_filter)?;
                filter_sql.push(format!("({region_name_match} OR {region_code_match})"));
            }
            if let Some(country_filter) = &filter.country {
                let country_name_match = generate_sql_match_condition(country_name, country_filter)?;
                let country_code_match = generate_sql_match_condition(country_code, country_filter)?;
                filter_sql.push(format!("({country_name_match} OR {country_code_match})"));
            }
            if filter_sql.len() > 0 {
                query = query.where_or(format!("({})", filter_sql.join(" AND ")).as_str());
            }
        }
    }
    query = query
        .order_by(city_name)
        .order_by(region_name)
        .order_by(country_name)
        .limit(&limit.to_string());

    // execute the query
    let stmt = prepare_sql!(conn, &query.to_string(), "failed to prepare select SQL")?;
    let mut cities = Vec::new();
    query_rows(stmt, [], |row| {
        let city = City {
            country_name: row.get(country_name).unwrap(),
            country_code: row.get(country_code).unwrap(),
            region_name: row.get(region_name).unwrap(),
            region_code: row.get(region_code).unwrap(),
            name: row.get(city_name).unwrap(),
            latitude: row.get(CityTbl::Latitude.column()).unwrap(),
            longitude: row.get(CityTbl::Longitude.column()).unwrap(),
            tz: row.get(CityTbl::Tz.column()).unwrap(),
        };
        cities.push(city);
        Ok(())
    })?;
    Ok(cities)
}

/// Get details about the Cities database.
///
/// # Arguments
///
/// * `weather_dir` is the weather history data directory.
///
pub fn details(weather_dir: &WeatherDir) -> crate::Result<Option<CitiesDetails>> {
    let db_file = weather_dir.file(CITIES_DB_FILENAME);
    // get the database connection
    let conn = db_conn_if_ready(weather_dir)?;

    // execute the query
    let co = "co";
    let r = "r";
    let ci = "ci";
    let country_name = "country_name";
    let country_code = "country_code";
    let region_name = "region_name";
    let region_code = "region_code";
    let city_count = "city_count";
    let query = sql::Select::new()
        .select(&CountryTbl::Name.alias_column_as_name(co, country_name))
        .select(&CountryTbl::Code.alias_column_as_name(co, country_code))
        .select(&RegionTbl::Name.alias_column_as_name(r, region_name))
        .select(&RegionTbl::Code.alias_column_as_name(r, region_code))
        .select(&format!("COUNT(*) AS {city_count}"))
        .from(&CountryTbl::table_as(co))
        .inner_join(&RegionTbl::alias_join_country_as(r, co))
        .inner_join(&CityTbl::alias_join_region_as(ci, r))
        .group_by(country_name)
        .group_by(region_name)
        .order_by(country_name)
        .order_by(region_name)
        .to_string();
    let stmt = prepare_sql!(conn, &query, "failed to prepare details SQL")?;

    // collect the query details
    let mut countries: HashMap<CountryMD, Vec<RegionDetails>> = HashMap::new();
    query_rows(stmt, [], |row| {
        let country_md = CountryMD { name: row.get(country_name).unwrap(), code: row.get(country_code).unwrap() };
        let region_details = RegionDetails {
            name: row.get(region_name).unwrap(),
            code: row.get(region_code).unwrap(),
            city_count: row.get::<_, i64>(city_count).unwrap() as usize,
        };
        match countries.get_mut(&country_md) {
            Some(details) => details.push(region_details),
            None => {
                let _ = countries.insert(country_md, vec![region_details]);
            }
        }
        Ok(())
    })?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::{db::sqlite::tables, testlib};

    fn init() -> testlib::TestFixture {
        let fixture = testlib::TestFixture::create();
        fixture.copy_resources(&testlib::test_resources().join("cities"));
        let weather_dir = WeatherDir::new(PathBuf::from(&fixture)).unwrap();
        let conn = db_conn!(&weather_dir).unwrap();
        tables::cities::initialize_schema(&conn).unwrap();
        load(&weather_dir, weather_dir.file("uscities.csv").path().to_str().unwrap(), false).unwrap();
        load(&weather_dir, weather_dir.file("canadacities.csv").path().to_str().unwrap(), false).unwrap();
        fixture
    }

    #[test]
    fn get() {
        // initialize the test fixture
        let fixture = init();
        let weather_dir = WeatherDir::new(PathBuf::from(&fixture)).unwrap();

        // spot check get without filters
        let testcases = super::get(&weather_dir, None, 5).unwrap();
        assert_eq!(testcases.len(), 5);
        let testcase = testcases.first().unwrap();
        println!("{testcase:?}");
        assert_eq!(testcase.country_name, "United States");
        assert_eq!(testcase.country_code, "US");
        assert_eq!(testcase.region_name, "Georgia");
        assert_eq!(testcase.region_code, "GA");
        assert_eq!(testcase.name, "Atlanta");
        assert_eq!(testcase.latitude, "33.7628");
        assert_eq!(testcase.longitude, "-84.4220");
        assert_eq!(testcase.tz, "America/New_York");
        let testcase = testcases.last().unwrap();
        println!("{testcase:?}");
        assert_eq!(testcase.country_name, "Canada");
        assert_eq!(testcase.country_code, "CA");
        assert_eq!(testcase.region_name, "Alberta");
        assert_eq!(testcase.region_code, "AB");
        assert_eq!(testcase.name, "Edmonton");
        assert_eq!(testcase.latitude, "53.5344");
        assert_eq!(testcase.longitude, "-113.4903");
        assert_eq!(testcase.tz, "America/Edmonton");

        // spot check a country filter
        let testcase = super::get(&weather_dir, Some(vec![LocationFilter::country("us")]), 2).unwrap();
        assert_eq!(testcase.len(), 2);
        assert_eq!(testcase.first().unwrap().name, "Atlanta");
        assert_eq!(testcase.last().unwrap().name, "Chicago");

        // spot check a country filter
        let testcase = super::get(&weather_dir, Some(vec![LocationFilter::country("ca")]), 2).unwrap();
        // testcase.iter().for_each(|city| println!("{city:?}"));
        assert_eq!(testcase.len(), 2);
        assert_eq!(testcase.first().unwrap().name, "Calgary");
        assert_eq!(testcase.last().unwrap().name, "Edmonton");

        // spot check a country filter
        let testcase = super::get(&weather_dir, Some(vec![LocationFilter::region("Ontario")]), 2).unwrap();
        assert_eq!(testcase.len(), 2);
        assert_eq!(testcase.first().unwrap().name, "Hamilton");
        assert_eq!(testcase.last().unwrap().name, "Ottawa");

        // spot check a city filter
        let testcase = super::get(&weather_dir, Some(vec![LocationFilter::city("w*")]), 2).unwrap();
        assert_eq!(testcase.len(), 2);
        assert_eq!(testcase.first().unwrap().name, "Washington");
        assert_eq!(testcase.last().unwrap().name, "Winnipeg");

        // spot multiple filters
        let filters = vec![
            LocationFilter::city("Philadelphia").with_region("PA").with_country("United States"),
            LocationFilter::city("Vancouver").with_region("British Columbia").with_country("ca"),
        ];
        let testcase = super::get(&weather_dir, Some(filters), 10).unwrap();
        assert_eq!(testcase.len(), 2);
        assert_eq!(testcase.first().unwrap().name, "Philadelphia");
        assert_eq!(testcase.last().unwrap().name, "Vancouver");
    }

    #[test]
    fn details() {
        // initialize the test fixture
        let fixture = init();
        let weather_dir = WeatherDir::new(PathBuf::from(&fixture)).unwrap();

        let testcases_opt = super::details(&weather_dir).unwrap();
        let testcases = testcases_opt.unwrap();
        assert!(testcases.db_size > 512); // the db has at least one page and 512 is the minimum size of a page
        assert_eq!(testcases.country_details.len(), 2);

        let testcase = testcases.country_details.first().unwrap();
        assert_eq!(testcase.region_details.len(), 5);
        assert_eq!(testcase.region_details[1].name, "British Columbia");
        assert_eq!(testcase.region_details[1].code, "BC");
        assert_eq!(testcase.region_details[1].city_count, 1);
        assert_eq!(testcase.region_details[3].name, "Ontario");
        assert_eq!(testcase.region_details[3].code, "ON");
        assert_eq!(testcase.region_details[3].city_count, 3);

        let testcase = testcases.country_details.last().unwrap();
        assert_eq!(testcase.region_details.len(), 8);
        assert_eq!(testcase.region_details[2].name, "Florida");
        assert_eq!(testcase.region_details[2].code, "FL");
        assert_eq!(testcase.region_details[2].city_count, 1);
        assert_eq!(testcase.region_details[7].name, "Texas");
        assert_eq!(testcase.region_details[7].code, "TX");
        assert_eq!(testcase.region_details[7].city_count, 2);
    }
}
