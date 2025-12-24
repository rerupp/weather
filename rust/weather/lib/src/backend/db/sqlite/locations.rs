//! The Sqlite implementation for locations.

use crate::{
    admin_prelude::DbLocationProblems,
    backend::{
        db::sqlite::{commit_tx, create_tx, execute_sql, prepare_sql, query_rows, SqlResult},
        filesys::{fs_lib, WeatherDir},
    },
    entities::{Location, LocationFilter},
};
use rusqlite::{named_params, Connection, Row, Transaction};
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

/// Add a location to both the filesystem and database. If there is an error updating the location
/// file the database will not be updated.
///
/// # Arguments
///
/// * `conn` is the database connection that will be used.
/// * `location` is what will be added.
/// * `weather_dir` is the directory containing the location file.
///
pub fn add(conn: &mut Connection, location: Location, weather_dir: &WeatherDir) -> crate::Result<()> {
    // add the location to the filesys first
    let location = fs_lib::add_location(weather_dir, location)?;

    // not sure that you need a transaction here but use one anyway
    let tx = create_tx!(conn, "failed getting transaction")?;
    add_db(&tx, location)?;
    commit_tx!(tx, "failed adding location to database")
}

/// Add a location to the database.
///
/// # Arguments
///
/// * `tx` is the database transaction that will be used.
/// * `location` is what will be added.
///
pub fn add_db(tx: &Transaction, location: Location) -> crate::Result<()> {
    const SQL: &str = r#"
        INSERT INTO locations (city, state, state_id, alias, latitude, longitude, tz)
            VALUES (:city, :state, :state_id, :alias, :latitude, :longitude, :tz)
        "#;
    let mut stmt = prepare_sql!(tx, SQL, "failed to prepare insert SQL")?;
    let alias = location.alias.clone();
    let params = named_params! {
        ":city": location.city,
        ":state": location.state,
        ":state_id": location.state_id,
        ":alias": location.alias,
        ":latitude": location.latitude,
        ":longitude": location.longitude,
        ":tz": location.tz,
    };
    execute_sql!(stmt, params, "'{alias}' location was not added")?;
    Ok(())
}

pub fn update(conn: &mut Connection, location: Location, weather_dir: &WeatherDir) -> crate::Result<bool> {
    match fs_lib::update_location(weather_dir, location)? {
        None => Ok(false),
        Some(location) => {
            // the location coming back from fs_lib will have sanitized property values
            let tx = create_tx!(conn, "failed to get update transaction")?;
            update_db(&tx, &location)?;
            commit_tx!(tx, "failed updating {location} in database")?;
            Ok(true)
        }
    }
}

/// Update a location in the database.
///
/// # Arguments
///
/// * `tx` is the database transaction that will be used.
/// * `location` has the new properties that will be persisted.
///
pub fn update_db(tx: &Transaction, location: &Location) -> crate::Result<bool> {
    // get the db location
    let mut locations = get(tx, Some(vec![LocationFilter::name(&location.alias)]))?;
    let db_location = match locations.len() {
        1 => locations.pop().unwrap(),
        len => {
            if len == 0 {
                log::error!("Did not find {} ({}) in the database.", location.name, location.alias);
            } else {
                log::error!("Found {} locations for {} ({}) in the database.", len, location.name, location.alias);
            }
            return Ok(false);
        }
    };

    // make sure there are changes to be applied
    let mut changes = vec![];
    macro_rules! update_if_changed {
        ($what: literal, $attr: ident) => {{
            let $attr = location.$attr.trim();
            if !$attr.is_empty() && $attr != db_location.$attr {
                changes.push(format!("{} = '{}'", $what, $attr));
            }
        }};
    }
    update_if_changed!("city", city);
    update_if_changed!("state", state);
    update_if_changed!("state_id", state_id);
    update_if_changed!("latitude", latitude);
    update_if_changed!("longitude", longitude);
    update_if_changed!("tz", tz);
    if changes.is_empty() {
        log::debug!("There are no changes to {db_location} properties.");
        return Ok(false);
    }

    // update the location
    let mut update = sql::Update::new().update("locations").where_clause(&format!("alias='{}'", location.alias));
    for change in &changes {
        update = update.set(change);
    }
    let update = update.to_string();
    let mut stmt = prepare_sql!(tx, &update, "failed to prepare '{update}'")?;
    let updates = execute_sql!(stmt, [], "error updating {}", location.alias)?;
    Ok(updates == 1)
}

/// The `DbAdmin` API uses this function to delete a location from the database and backing store.
///
/// # Arguments
///
/// * `tx` is the database transaction that will be used.
/// * `alias` is the location alias name.
/// * `weather_dir` is the weather history data directory.
///
pub fn delete(tx: &Transaction, alias: &str, weather_dir: &WeatherDir) -> crate::Result<bool> {
    // always delete the location from the backing store first
    fs_lib::delete_location(weather_dir, alias)?;
    delete_db(tx, alias)
}

/// Used internally to delete a location from the database.
///
/// # Arguments
///
/// * `tx` is the database transaction that will be used.
/// * `alias` is the location alias name.
///
fn delete_db(tx: &Transaction, alias: &str) -> crate::Result<bool> {
    static DELETE: &str = "DELETE FROM locations WHERE alias=:alias";
    let mut stmt = prepare_sql!(tx, DELETE, "failed to prepare delete SQL")?;
    let deletes = execute_sql!(stmt, named_params! { ":alias": alias }, "'{alias}' was not deleted")?;
    Ok(deletes == 1)
}

/// Get the weather data locations.
///
/// # Arguments
///
/// * `conn` is the database connection that will be used.
/// * `filters` determines what locations will be returned.
///
pub fn get(conn: &Connection, filters: Option<Vec<LocationFilter>>) -> crate::Result<Vec<Location>> {
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

fn get_query(optional_filters: Option<Vec<LocationFilter>>) -> String {
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
    if let Some(filters) = optional_filters {
        for filter in filters {
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
                    query = query.where_or(&format!(
                        "({} AND {} AND {})",
                        like_city(city),
                        like_state(state),
                        like_name(name)
                    ));
                }
                _ => (),
            }
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

/// Loads the filesystem locations document into the database.
///
/// # Arguments
///
/// * `conn` is the database connection that will be used.
/// * `weather_dir` is the weather data directory.
///
pub fn load(conn: &mut Connection, weather_dir: &WeatherDir) -> crate::Result<()> {
    let mut insert =
        sql::Insert::new().insert_into("locations (city, state, state_id, alias, latitude, longitude, tz)");
    let locations = fs_lib::get_locations(weather_dir, None)?;
    if locations.is_empty() {
        return Ok(());
    }
    for location in locations {
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

/// Compare the database locations against the filesystem locations document.
///
/// # Arguments
///
/// * `conn` is the database connection that will be used.
/// * `weather_dir` is the weather data directory.
///
pub fn check(conn: &mut Connection, weather_dir: &WeatherDir) -> crate::Result<Option<DbLocationProblems>> {
    let fs_locations = fs_lib::get_locations(weather_dir, None)?;
    let db_locations = get(conn, None)?;

    // find any missing locations
    macro_rules! find_missing {
        ($lhs:expr, $rhs:expr) => {
            $lhs.iter()
                .filter(|lhs| !$rhs.iter().any(|rhs| &lhs.alias == &rhs.alias))
                .map(|lhs| lhs.clone())
                .collect::<Vec<_>>()
        };
    }
    let missing_locations = find_missing!(fs_locations, db_locations);
    let detached_locations = find_missing!(db_locations, fs_locations);

    match missing_locations.is_empty() && detached_locations.is_empty() {
        true => Ok(None),
        false => {
            let mut problems = DbLocationProblems::default();
            if missing_locations.len() > 0 {
                problems.missing_locations.replace(missing_locations);
            }
            if detached_locations.len() > 0 {
                problems.detached_locations.replace(detached_locations);
            }
            Ok(Some(problems))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::{
        db::{
            admin::DbAdmin,
            sqlite::{admin::SQLiteAdmin, commit_tx, create_tx, db_conn},
        },
        testlib, WeatherDir,
    };
    use std::rc::Rc;

    macro_rules! assert_locations {
        ($lhs:expr, $rhs:expr) => {
            assert_eq!($lhs.city, $rhs.city);
            assert_eq!($lhs.state, $rhs.state);
            assert_eq!($lhs.state_id, $rhs.state_id);
            assert_eq!($lhs.latitude, $rhs.latitude);
            assert_eq!($lhs.longitude, $rhs.longitude);
            assert_eq!($lhs.tz, $rhs.tz);
        };
    }

    fn init_fixture(copy_resources: bool) -> testlib::TestFixture {
        let fixture = testlib::TestFixture::create();
        if copy_resources {
            let mut resources = testlib::test_resources();
            resources.push("db");
            fixture.copy_resources(&resources);
        }
        // initialize the database schema
        let weather_dir = Rc::new(WeatherDir::from(&fixture));
        SQLiteAdmin::new(weather_dir).history_init(false).unwrap();
        fixture
    }

    #[test]
    fn add_delete() {
        let fixture = init_fixture(false);
        let weather_dir = WeatherDir::from(&fixture);
        let mut conn = db_conn!(weather_dir).unwrap();

        // verify the initial state
        assert!(fs_lib::get_locations(&weather_dir, None).unwrap().is_empty());
        assert!(get(&conn, None).unwrap().is_empty());

        let alias = "foothills";
        // add a location and verify the results
        let added_location = Location {
            city: "Fortuna Foothills".to_string(),
            state_id: "AZ".to_string(),
            state: "Arizona".to_string(),
            name: Default::default(),
            alias: alias.to_string(),
            latitude: "32.6578355".to_string(),
            longitude: "-114.4118901".to_string(),
            tz: "America/Phoenix".to_string(),
        };
        add(&mut conn, added_location.clone(), &weather_dir).unwrap();
        let added_fs_locations = fs_lib::get_locations(&weather_dir, None).unwrap();
        assert_eq!(added_fs_locations.len(), 1);
        assert_locations!(added_location, &added_fs_locations[0]);
        let added_db_locations = get(&conn, None).unwrap();
        assert_eq!(added_db_locations.len(), 1);
        assert_locations!(added_location, &added_db_locations[0]);

        // update the location and verify the results
        let updated_location = Location {
            city: "Yuma".to_string(),
            state_id: "".to_string(),
            state: "".to_string(),
            name: "".to_string(),
            alias: alias.to_string(),
            latitude: "".to_string(),
            longitude: "".to_string(),
            tz: "".to_string(),
        };
        assert!(update(&mut conn, updated_location.clone(), &weather_dir).unwrap());

        // get the fs locations and verify
        let updated_fs_locations = fs_lib::get_locations(&weather_dir, None).unwrap();
        assert_eq!(updated_fs_locations.len(), 1);
        let updated_fs_location = &updated_fs_locations[0];
        assert_eq!(updated_fs_location.city, updated_location.city);
        assert_eq!(updated_fs_location.state_id, added_location.state_id);
        assert_eq!(updated_fs_location.state, added_location.state);
        assert_eq!(updated_fs_location.latitude, added_location.latitude);
        assert_eq!(updated_fs_location.longitude, added_location.longitude);
        assert_eq!(updated_fs_location.tz, added_location.tz);

        // the db location should match the fs location
        let updated_db_locations = get(&conn, None).unwrap();
        assert_eq!(updated_db_locations.len(), 1);
        assert_locations!(updated_fs_location, &updated_db_locations[0]);

        // delete the location and verify the results
        let tx = create_tx!(conn, "failed to create delete transaction").unwrap();
        assert!(delete(&tx, alias, &weather_dir).unwrap());
        commit_tx!(tx, "error deleting location '{alias}'").unwrap();
        assert!(fs_lib::get_locations(&weather_dir, None).unwrap().is_empty());
        assert!(get(&conn, None).unwrap().is_empty());

        // now make sure the delete or not is coming back
        let tx = create_tx!(conn, "failed to create delete transaction").unwrap();
        assert!(!delete(&tx, alias, &weather_dir).unwrap());
    }

    #[test]
    fn add_update_delete_db() {
        let fixture = init_fixture(true);
        let south = "south";
        let south_filter = || Some(vec![LocationFilter::name(south)]);

        // get south from the locations document
        let weather_dir = WeatherDir::from(&fixture);
        let fs_location = fs_lib::get_locations(&weather_dir, south_filter()).unwrap().remove(0);

        // add the location to the database
        let mut conn = db_conn!(weather_dir).unwrap();
        let tx = create_tx!(conn, "failed getting add transaction").unwrap();
        add_db(&tx, fs_location.clone()).unwrap();
        commit_tx!(tx, "failed adding location to test database").unwrap();

        // get the location from the db and make sure it's what you expect
        let db_locations = get(&conn, south_filter()).unwrap();
        assert_eq!(db_locations.len(), 1);
        // let db_location = &db_locations[0];
        assert_locations!(fs_location, db_locations[0]);

        // don't update a location with the same properties
        let tx = create_tx!(conn, "failed getting update transaction").unwrap();
        assert!(!update_db(&tx, &fs_location).unwrap());

        // make the changes
        let update = Location {
            city: "city".to_string(),
            state_id: "id".to_string(),
            state: "state".to_string(),
            name: Default::default(),
            alias: south.to_string(),
            latitude: "12.345".to_string(),
            longitude: "54.321".to_string(),
            tz: "utc".to_string(),
        };
        assert!(update_db(&tx, &update).unwrap());
        commit_tx!(tx, "failed updating location in db").unwrap();

        // make sure the update is what you expect
        let updated_locations = get(&conn, south_filter()).unwrap();
        assert_eq!(updated_locations.len(), 1);
        assert_locations!(update, updated_locations[0]);

        // check a partial update
        let partial = Location {
            city: "Some City".to_string(),
            state_id: "XX".to_string(),
            state: "Xerces".to_string(),
            name: Default::default(),
            alias: south.to_string(),
            latitude: "".to_string(),
            longitude: "".to_string(),
            tz: "".to_string(),
        };
        let tx = create_tx!(conn, "failed getting partial update transaction").unwrap();
        assert!(update_db(&tx, &partial).unwrap());
        commit_tx!(tx, "failed partial update of location in db").unwrap();
        let partial_updates = get(&conn, south_filter()).unwrap();
        assert_eq!(partial_updates.len(), 1);
        let partial_update = &partial_updates[0];
        assert_eq!(partial_update.city, partial.city);
        assert_eq!(partial_update.state_id, partial.state_id);
        assert_eq!(partial_update.state, partial.state);
        assert_eq!(partial_update.latitude, update.latitude);
        assert_eq!(partial_update.longitude, update.longitude);
        assert_eq!(partial_update.tz, update.tz);

        // delete the location you just added
        let tx = create_tx!(conn, "failed getting delete transaction").unwrap();
        assert!(delete_db(&tx, south).unwrap());
        assert!(!delete_db(&tx, south).unwrap());
        commit_tx!(tx, "failed deleting location in db").unwrap();
        assert!(get(&conn, None).unwrap().is_empty());
    }

    #[test]
    fn load_id_aliases() {
        // create the test environment
        let fixture = init_fixture(true);
        let weather_dir = WeatherDir::from(&fixture);

        // load the locations
        let mut conn = db_conn!(weather_dir).unwrap();
        load(&mut conn, &weather_dir).unwrap();
        let db_locations = get(&conn, None).unwrap();

        // verify the locations
        let fs_locations = fs_lib::get_locations(&weather_dir, None).unwrap();
        assert_eq!(db_locations.len(), fs_locations.len());
        for (lhs, rhs) in db_locations.iter().zip(fs_locations.iter()) {
            assert_eq!(lhs.city, rhs.city);
            assert_eq!(lhs.state, rhs.state);
            assert_eq!(lhs.state_id, rhs.state_id);
            assert_eq!(lhs.alias, rhs.alias);
            assert_eq!(lhs.latitude, rhs.latitude);
            assert_eq!(lhs.longitude, rhs.longitude);
            assert_eq!(lhs.tz, rhs.tz);
        }

        // verify the helpers that get location row ids
        let id_aliases = id_aliases(&conn).unwrap();
        assert_eq!(id_aliases.len(), db_locations.len());
        for (id, alias) in id_aliases {
            assert_eq!(id, location_id(&conn, &alias).unwrap());
        }
    }
}
