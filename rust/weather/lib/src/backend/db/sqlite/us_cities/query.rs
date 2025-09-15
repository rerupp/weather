/// The US Cities user queries.
///
use crate::{
    backend::db::sqlite::{err, prepare_sql, query_rows, SqlResult},
    entities::{CityFilter, Location, State},
    log_elapsed_time,
};
use rusqlite::{Connection, Row};
use sql_query_builder as sql;

/// Get a collection of US City metadata.
///
/// # Arguments
///
/// * `conn` is the database connection that will be used.
/// * `filter` is used to restrict which US Cities should be found.
/// * `limit` is the maximum number of locations.
///
pub fn cities(conn: &Connection, filter: CityFilter) -> crate::Result<Vec<Location>> {
    let elapsed_query = crate::LogElapsedTime::new("locations: query", Some(log::Level::Trace));
    let query = build_query(filter);
    // println!("{query}");
    let mut stmt = prepare_sql!(conn, &query, "failed to prepare query")?;
    let mut rows = query_rows!(stmt, [], "error executing query")?;
    drop(elapsed_query);

    log_elapsed_time!("locations: create");
    let mut locations = vec![];
    loop {
        match rows.next() {
            Err(error) => err!("failed getting next row: {:?}", error)?,
            Ok(None) => break,
            Ok(Some(row)) => {
                #[inline]
                fn next_location(row_: &Row) -> SqlResult<Location> {
                    let city: String = row_.get("city")?;
                    let state_id: String = row_.get("state_id")?;
                    Ok(Location {
                        name: format!("{city}, {state_id}"),
                        city,
                        state_id,
                        state: row_.get("state")?,
                        alias: Default::default(),
                        latitude: row_.get("latitude")?,
                        longitude: row_.get("longitude")?,
                        tz: row_.get("timezone")?,
                    })
                }
                match next_location(&row) {
                    Ok(location) => locations.push(location),
                    Err(error) => err!("error creating location from row: {:?}", error)?,
                }
            }
        }
    }
    Ok(locations)
}

/// Create the query used to get the US Cities data.
///
/// # Arguments
///
/// * `filter` is the optional city filters.
/// * `limit` is the number of cities to return.
///
fn build_query(filter: CityFilter) -> String {
    const SELECT_CLAUSE: &str = "
        DISTINCT
        cities.city AS city,
        states.name AS state,
        states.state_id AS state_id,
        cities.latitude AS latitude,
        cities.longitude AS longitude,
        cities.timezone AS timezone
    ";
    let mut query = sql::Select::new()
        .select(SELECT_CLAUSE)
        .from("cities")
        .inner_join("states ON cities.states_id=states.id")
        .inner_join("city_zip_codes ON cities.id=city_zip_codes.cities_id");
    if let Some(mut city) = filter.name {
        if city.contains('*') {
            city = city.replace('*', "%");
        }
        query = query.where_and(&format!("city LIKE '{city}'"));
    }
    if let Some(mut state) = filter.state {
        if state.contains('*') {
            state = state.replace("*", "%");
        };
        query = query.where_and(&format!("(state LIKE '{state}' OR state_id LIKE '{state}')"));
    }
    if let Some(mut zip_code) = filter.zip_code {
        if zip_code.contains('*') {
            zip_code = zip_code.replace("*", "%");
        }
        query = query
            .inner_join("zip_codes ON city_zip_codes.zip_codes_id=zip_codes.id")
            .where_and(&format!("zip_codes.zip_code LIKE '{zip_code}'"));
    }

    query.order_by("city, state_id").limit(&format!("{}", filter.limit)).to_string()
}

/// Get the collection of US City states.
///
/// # Arguments
///
/// * `conn` is the database connection that will be used.
///
pub fn states(conn: &Connection) -> crate::Result<Vec<State>> {
    const QUERY: &str = "SELECT name, state_id FROM states ORDER BY name";
    let mut stmt = prepare_sql!(conn, QUERY, "failed to prepare states query")?;
    let mut rows = query_rows!(stmt, [], "error executing query")?;
    let mut states = vec![];
    loop {
        match rows.next() {
            Err(error) => err!("failed getting next state row: {:?}", error)?,
            Ok(None) => break,
            Ok(Some(row)) => {
                #[inline]
                fn next_state(row_: &Row) -> SqlResult<State> {
                    Ok(State { name: row_.get("name")?, state_id: row_.get("state_id")? })
                }
                match next_state(row) {
                    Ok(state) => states.push(state),
                    Err(error) => err!("error creating state from row: {:?}", error)?,
                }
            }
        }
    }
    Ok(states)
}
