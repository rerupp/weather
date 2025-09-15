//! The Sqlite implementation for locations.

use crate::{
    backend::{
        db::sqlite::{create_tx, execute_sql, prepare_sql, query_rows, SqlResult},
        filesys::{self, WeatherDir}
    },
    entities::{Location, LocationFilters},
};
use rusqlite::{named_params, Connection, Row};
use sql_query_builder as sql;

/// Create a database locations specific error message.
macro_rules! error {
    ($($arg:tt)*) => {
        crate::Error::from(format!("DB Locations {}", format!($($arg)*)))
    }
}

/// Create an error from the locations specific error message.
macro_rules! err {
    ($($arg:tt)*) => {
        Err(error!($($arg)*))
    };
}

/// Add a location to the location file and database. If there is an error updating the location
/// file the database will not be updated.
///
/// # Arguments
///
/// * `conn` is the database connection that will be used.
/// * `location` is what will be added.
/// * `weather_dir` is the directory containing the location file.
///
pub fn add(conn: &mut Connection, mut location: Location, weather_dir: &WeatherDir) -> crate::Result<()> {
    // add the location to the filesys first
    let locations = filesys::Locations::open(weather_dir)?;
    location = locations.add(location)?;

    // not sure that you need a transaction here but use one anyway
    let tx = create_tx!(conn, "failed getting transaction")?;
    const SQL: &str = r#"
        INSERT INTO locations (city, state, state_id, alias, latitude, longitude, tz)
            VALUES (:city, :state, :state_id, :alias, :latitude, :longitude, tz)
        "#;
    let mut stmt = prepare_sql!(tx, SQL, "failed to prepare insert SQL")?;
    let alias = location.alias.clone();
    let params = named_params! {
        ":city": location.city,
        ":state": location.state,
        ":state_id": location.state,
        ":alias": location.alias,
        ":latitude": location.latitude,
        ":longitude": location.longitude,
        ":tz": location.tz,
    };
    execute_sql!(stmt, params, "'{alias}' location was not added")
}

/// Get the weather data locations.
///
/// # Arguments
///
/// * `conn` is the database connection that will be used.
/// * `filters` determines what locations will be returned.
///
pub fn get(conn: &Connection, filters: LocationFilters) -> crate::Result<Vec<Location>> {
    // run the query
    let sql = get_query(filters);
    let mut stmt = prepare_sql!(conn, &sql, "failed to prepare query SQL")?;
    let mut rows = query_rows!(stmt, [], "failed to execute query")?;

    let mut locations = vec![];
    loop {
        let row = match rows.next() {
            Ok(Some(row)) => row,
            Ok(None) => break,
            Err(error) => err!("failed to execute query: {:?}", error)?,
        };
        fn next_location(row_: &Row) -> SqlResult<Location> {
            let city: String = row_.get("city")?;
            let state_id: String = row_.get("state_id")?;
            Ok(Location {
                name: format!("{}, {}", city, state_id),
                city,
                state_id,
                state: row_.get("state")?,
                alias: row_.get("alias")?,
                longitude: row_.get("longitude")?,
                latitude: row_.get("latitude")?,
                tz: row_.get("tz")?,
            })
        }
        match next_location(row) {
            Ok(location) => locations.push(location),
            Err(error) => err!("failed to create location from row: {:?}", error)?,
        }
    }
    Ok(locations)
}

fn get_query(location_filters: LocationFilters) -> String {
    #[inline]
    fn like_city(value: &str) -> String {
        format!("city LIKE '{}'", value.replace("*", "%"))
    }
    #[inline]
    fn like_state(state: &str) -> String {
        let state = state.replace("*", "%");
        format!("(state LIKE '{state}' OR state_id LIKE '{state}')")
    }
    #[inline]
    fn like_name(name: &str) -> String {
        let name = name.replace("*", "%");
        format!("(name LIKE '{name}' OR alias LIKE '{name}')")
    }
    let mut query =
        sql::Select::new().from("locations").select("city, state, state_id, alias, latitude, longitude, tz");
    for filter in location_filters {
        match (&filter.city, &filter.state, &filter.name) {
            (Some(city), None, None) => {
                query = query.where_or(&like_city(city));
            }
            (None, Some(state), None) => {
                query = query.where_or(&like_state(state));
            }
            (None, None, Some(name)) => {
                query = query.where_or(&like_name(name));
            }
            (Some(city), Some(state), None) => {
                query = query.where_or(&format!("({} AND {})", like_city(city), like_state(state)));
            }
            (Some(city), None, Some(name)) => {
                query = query.where_or(&format!("({} AND {})", like_city(city), like_name(name)));
            }
            (None, Some(state), Some(name)) => {
                query = query.where_or(&format!("({} AND {})", like_state(state), like_name(name)));
            }
            (Some(city), Some(state), Some(name)) => {
                query =
                    query.where_or(&format!("({} AND {} AND {})", like_city(city), like_state(state), like_name(name)));
            }
            _ => (),
        }
    }
    query.order_by("city, state_id ASC").to_string()
}

/// Get the location id and alias.
///
/// # Arguments
///
/// * `conn` is the database connection that will be used.
///
pub fn id_aliases(conn: &Connection) -> crate::Result<Vec<(i64, String)>> {
    // run the query
    const SQL: &'static str = "SELECT id, alias FROM locations ORDER BY alias ASC";
    let mut stmt = prepare_sql!(conn, SQL, "failed to prepare id_aliases SQL")?;
    let mut rows = query_rows!(stmt, [], "failed to execute id_aliases query")?;

    let mut id_aliases = vec![];
    loop {
        let row = match rows.next() {
            Ok(Some(row)) => row,
            Ok(None) => break,
            Err(error) => err!("failed to next id_aliases row: {:?}", error)?,
        };
        fn next_id_alias(row_: &Row) -> SqlResult<(i64, String)> {
            Ok((row_.get(0)?, row_.get(1)?))
        }
        match next_id_alias(row) {
            Ok(id_alias) => id_aliases.push(id_alias),
            Err(error) => err!("failed to get id and alias: {:?}", error)?,
        }
    }
    Ok(id_aliases)
}

/// Get the locations database identifier.
///
/// # Arguments
///
/// * `conn` is the database connection that will be used.
/// * `alias` is the location alias name.
///
pub fn location_id(conn: &Connection, alias: &str) -> crate::Result<i64> {
    const SQL: &'static str = "SELECT id FROM locations AS l WHERE l.alias = :alias";
    let mut stmt = prepare_sql!(conn, SQL, "failed to prepare location_id sql")?;
    match stmt.query_row(named_params! {":alias": alias}, |row| Ok(row.get(0))) {
        Err(error) => {
            err!("failed to find location id for '{}', {:?}", alias, error)
        }
        Ok(id_result) => match id_result {
            Ok(id) => Ok(id),
            Err(error) => err!("failed to get location id for '{}', {:?}", alias, error),
        },
    }
}

/// Loads the location file into the database.
///
/// # Arguments
///
/// * `conn` is the database connection that will be used.
/// * `weather_dir` is the weather data directory.
///
pub fn load(conn: &mut Connection, weather_dir: &WeatherDir) -> crate::Result<()> {
    let mut insert =
        sql::Insert::new().insert_into("locations (city, state, state_id, alias, latitude, longitude, tz)");
    for location in filesys::Locations::open(weather_dir)?.get()? {
        insert = insert.values(&format!(
            "('{}', '{}', '{}', '{}', '{}', '{}', '{}')",
            location.city,
            location.state,
            location.state_id,
            location.alias,
            location.latitude,
            location.longitude,
            location.tz
        ));
    }
    let rows_inserted = match conn.execute(&insert.to_string(), []) {
        Ok(rows_inserted) => rows_inserted,
        Err(error) => err!("failed to insert the locations: {:?}", error)?,
    };
    log::debug!("{} locations added.", rows_inserted);
    Ok(())
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use crate::backend::db::sqlite::db_conn;
    // use std::path::PathBuf;
    //
    // #[test]
    // fn load() {
    //     let weather_dir = WeatherDir::new(PathBuf::from("../weather_data")).unwrap();
    //     let mut conn = db_conn!(&weather_dir).unwrap();
    //     super::load(&mut conn, &weather_dir).unwrap();
    // }
    //
    //     #[test]
    //     fn get() {
    //         let weather_dir = WeatherDir::new(PathBuf::from("../weather_data")).unwrap();
    //         let conn = db_conn!(&weather_dir).unwrap();
    //         println!("all");
    //         for location in super::get(&conn, &vec![]).unwrap() {
    //             println!("{:?}", location);
    //         }
    //         let filters = vec!["d*".to_string(), "k*".to_string(), "z*".to_string()];
    //         println!("{:?}", filters);
    //         for location in super::get(&conn, &filters).unwrap() {
    //             println!("{:?}", location);
    //         }
    //     }
    //
    //     #[test]
    //     fn id_aliases() {
    //         let weather_dir = WeatherDir::new(PathBuf::from("../weather_data")).unwrap();
    //         let conn = db_conn!(&weather_dir).unwrap();
    //         let id_aliases = super::id_aliases(&conn).unwrap();
    //         println!("{:?}", id_aliases);
    //     }
    //
    //     #[test]
    //     fn location_id() {
    //         let weather_dir = WeatherDir::new(PathBuf::from("../weather_data")).unwrap();
    //         let conn = db_conn!(&weather_dir).unwrap();
    //         let id = super::location_id(&conn, "kfalls").unwrap();
    //         println!("kfalls id = {id}");
    //     }
    //
    // #[test]
    // fn query() {
    //     println!(
    //         "{}",
    //         get_query(&vec![
    //             LocationFilter::default().with_city("Tigard").with_state("OR"),
    //             LocationFilter::default(). with_state("OR").with_name("kfalls"),
    //         ])
    //     );
    // }
}
