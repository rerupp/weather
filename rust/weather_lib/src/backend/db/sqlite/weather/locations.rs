//! The Sqlite implementation for locations.

use crate::{
    admin_prelude::DbLocationProblems,
    backend::{
        db::sqlite::{
            commit_tx, create_tx, execute_sql, generate_sql_match_condition, prepare_cached_sql, prepare_sql,
            query_rows,
            tables::{named_param, weather::LocationsTbl, TblSqlBuilder},
        },
        filesys::{fs_lib, WeatherDir},
    },
    entities::{Location, LocationFilter},
};
use rusqlite::{Connection, Transaction};
use sql_query_builder as sql;

/// Create an error from the locations specific error message.
macro_rules! err {
    ($($arg:tt)*) => {
        Err(crate::Error(format!("DB Locations {}", format!($($arg)*))))
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
    let mut stmt = prepare_cached_sql!(tx, &LocationsTbl::insert_sql(), "failed to prepare insert SQL")?;
    let location_name = location.to_string();
    let values = [
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
    execute_sql!(stmt, &values, "'{location_name}' location was not added")?;
    Ok(())
}

/// Update a location properties.
///
/// # Arguments
///
/// * `conn` is the database connection that will be used.
/// * `location` contains the location updates.
/// * `weather_dir` is the weather history data directory.
///
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
    let mut locations = get(tx, Some(vec![LocationFilter::alias(&location.alias)]))?;
    let db_location = match locations.len() {
        1 => locations.pop().unwrap(),
        len => {
            if len == 0 {
                log::error!("Did not find {location} in the database.");
            } else {
                log::error!("Found {len} locations for {location} in the database.");
            }
            return Ok(false);
        }
    };

    macro_rules! column_value {
        ($column: path, $value: expr) => {
            format!("{}='{}'", $column.column(), $value)
        };
    }
    // make sure there are changes to be applied
    let mut changes = vec![];
    macro_rules! update_if_changed {
        ($column: path, $attr: ident) => {{
            let attr = location.$attr.trim();
            if !attr.is_empty() && attr != db_location.$attr {
                changes.push(column_value!($column, attr));
            }
        }};
    }
    // todo: make this use named_parameters
    update_if_changed!(LocationsTbl::CountryName, country_name);
    update_if_changed!(LocationsTbl::CountryCode, country_code);
    update_if_changed!(LocationsTbl::RegionName, region_name);
    update_if_changed!(LocationsTbl::RegionCode, region_code);
    update_if_changed!(LocationsTbl::CityName, city_name);
    update_if_changed!(LocationsTbl::Latitude, latitude);
    update_if_changed!(LocationsTbl::Longitude, longitude);
    update_if_changed!(LocationsTbl::Tz, tz);
    if changes.is_empty() {
        log::debug!("There are no changes to {db_location} properties.");
        return Ok(false);
    }

    // update the location
    let mut update = sql::Update::new()
        .update(LocationsTbl::TABLE)
        .where_clause(&column_value!(LocationsTbl::Alias, location.alias));
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
    let query = sql::Delete::new()
        .delete_from(LocationsTbl::TABLE)
        .where_clause(&format!("{}='{alias}'", LocationsTbl::Alias.column()))
        .as_string();
    let mut stmt = prepare_sql!(tx, &query, "failed to prepare delete SQL")?;
    let deletes = execute_sql!(stmt, [], "'{alias}' was not deleted")?;
    Ok(deletes == 1)
}

/// Get the weather data locations.
///
/// # Arguments
///
/// * `conn` is the database connection that will be used.
/// * `filters` determines what locations will be returned.
///
pub fn get(conn: &Connection, filters_opt: Option<Vec<LocationFilter>>) -> crate::Result<Vec<Location>> {
    // run the query
    let mut query = sql::Select::new().from(LocationsTbl::TABLE).select("*");
    if let Some(filters) = filters_opt {
        query = add_location_filters(query, &filters, None);
    }
    let sql = query
        .order_by(&LocationsTbl::CityName.column_asc())
        .order_by(&LocationsTbl::RegionCode.column_asc())
        .order_by(&LocationsTbl::CountryCode.column_asc())
        .order_by(&LocationsTbl::Alias.column_asc())
        .to_string();
    let stmt = prepare_sql!(conn, &sql, "failed to prepare query SQL")?;

    let mut locations = vec![];
    query_rows(stmt, [], |row| {
        let location = Location {
            country_name: row.get(LocationsTbl::CountryName.column()).unwrap(),
            country_code: row.get(LocationsTbl::CountryCode.column()).unwrap(),
            region_name: row.get(LocationsTbl::RegionName.column()).unwrap(),
            region_code: row.get(LocationsTbl::RegionCode.column()).unwrap(),
            city_name: row.get(LocationsTbl::CityName.column()).unwrap(),
            alias: row.get(LocationsTbl::Alias.column()).unwrap(),
            latitude: row.get(LocationsTbl::Latitude.column()).unwrap(),
            longitude: row.get(LocationsTbl::Longitude.column()).unwrap(),
            tz: row.get(LocationsTbl::Tz.column()).unwrap(),
        };
        locations.push(location);
        Ok(())
    })?;
    Ok(locations)
}

/// Get a location.
///
/// # Arguments
///
/// * `conn` is the database connection that will be used.
/// * `filter` is used to get the location.
///
pub fn get_one(conn: &Connection, filter: LocationFilter) -> crate::Result<Option<Location>> {
    let mut locations = get(conn, Some(vec![filter]))?;
    match locations.len() {
        0 => Ok(None),
        1 => Ok(locations.pop()),
        _ => err!("Multiple locations were found."),
    }
}

/// A builder method that adds location filters to a query. Filters with errors are ignored but
/// will be logged as an error.
///
/// # Arguments
///
/// * `query` is the SQL [Select](sql::Select) query that will be updated.
/// * `filters` are added as SQL `OR` conditions.
/// * `alias_opt` will be used to alias location table columns when provided.
///
pub fn add_location_filters(
    mut query: sql::Select,
    filters: &Vec<LocationFilter>,
    alias_opt: Option<&str>,
) -> sql::Select {
    // internal helper to generate SQL for the columns name or aliased name
    let generate_sql = |column: LocationsTbl, filter: &str| -> Option<String> {
        let generate_result = match alias_opt {
            None => generate_sql_match_condition(column.column(), filter),
            Some(alias) => generate_sql_match_condition(column.alias_column(alias), filter),
        };
        match generate_result {
            Ok(sql) => Some(sql),
            Err(error) => {
                log::error!("'{column:?}' filter '{filter}' is illegal: {error}");
                None
            }
        }
    };
    for filter in filters {
        let mut filter_sql = vec![];
        if let Some(filter) = &filter.alias {
            if let Some(sql) = generate_sql(LocationsTbl::Alias, filter) {
                filter_sql.push(sql);
            }
        }
        if let Some(filter) = &filter.city {
            if let Some(sql) = generate_sql(LocationsTbl::CityName, filter) {
                filter_sql.push(sql);
            }
        }
        if let Some(filter) = &filter.region {
            if let Some(name_filter) = generate_sql(LocationsTbl::RegionName, filter) {
                if let Some(code_filter) = generate_sql(LocationsTbl::RegionCode, filter) {
                    filter_sql.push(format!("({name_filter} OR {code_filter})"))
                }
            }
        }
        if let Some(filter) = &filter.country {
            if let Some(name_filter) = generate_sql(LocationsTbl::CountryName, filter) {
                if let Some(code_filter) = generate_sql(LocationsTbl::CountryCode, filter) {
                    filter_sql.push(format!("({name_filter} OR {code_filter})"))
                }
            }
        }
        if filter_sql.len() > 0 {
            query = query.where_or(format!("({})", filter_sql.join(" AND ")).as_str());
        }
    }
    query
}

/// Get the location id and alias.
///
/// # Arguments
///
/// * `conn` is the database connection that will be used.
///
pub fn id_aliases(conn: &Connection) -> crate::Result<Vec<(i64, String)>> {
    // run the query
    let query = sql::Select::new()
        .select(LocationsTbl::Id.column())
        .select(LocationsTbl::Alias.column())
        .from(LocationsTbl::TABLE)
        .order_by(&LocationsTbl::Alias.column_asc())
        .to_string();
    let stmt = prepare_sql!(conn, &query, "failed to prepare id_aliases SQL")?;
    let mut id_aliases = vec![];
    query_rows(stmt, [], |row| {
        let id = row.get(LocationsTbl::Id.column()).unwrap();
        let alias = row.get(LocationsTbl::Alias.column()).unwrap();
        id_aliases.push((id, alias));
        Ok(())
    })?;
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
    let query = sql::Select::new()
        .select(LocationsTbl::Id.column())
        .from(LocationsTbl::TABLE)
        .where_clause(&LocationsTbl::Alias.where_param())
        .to_string();
    let mut stmt = prepare_sql!(conn, &query, "failed to prepare location_id SQL")?;
    let params = [named_param!(LocationsTbl::Alias, alias)];
    match stmt.query_row(&params, |row| Ok(row.get(0))) {
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
    // get the locations from the filesystem
    let locations = fs_lib::get_locations(weather_dir, None)?;
    let locations_count = locations.len();
    if locations_count == 0 {
        return Ok(());
    }
    // insert the locations into the database
    let tx = create_tx!(conn, "failed to create load TX")?;
    for location in locations {
        add_db(&tx, location)?;
    }
    commit_tx!(tx, "failed to commit load TX")?;
    log::debug!("{locations_count} locations added.");
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
            sqlite::{admin::SQLiteAdmin, commit_tx, create_tx, weather},
        },
        testlib, WeatherDir,
    };

    macro_rules! assert_locations {
        ($lhs:expr, $rhs:expr) => {
            assert_eq!($lhs.country_name, $rhs.country_name);
            assert_eq!($lhs.country_code, $rhs.country_code);
            assert_eq!($lhs.region_name, $rhs.region_name);
            assert_eq!($lhs.region_code, $rhs.region_code);
            assert_eq!($lhs.city_name, $rhs.city_name);
            assert_eq!($lhs.alias, $rhs.alias);
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
        let weather_dir = WeatherDir::from(&fixture);
        SQLiteAdmin::new(weather_dir).history_init(false).unwrap();
        fixture
    }

    #[test]
    fn query() {
        let fixture = init_fixture(false);
        let weather_dir = WeatherDir::from(&fixture);
        let mut conn = weather::db_conn!(weather_dir).unwrap();

        macro_rules! location {
            ($country: expr, $region: expr, $city: expr, $alias: expr) => {
                Location {
                    country_name: "Country Name".into(),
                    country_code: $country.into(),
                    region_name: "Region Name".into(),
                    region_code: $region.into(),
                    city_name: $city.into(),
                    alias: $alias.into(),
                    latitude: "1".into(),
                    longitude: "1".into(),
                    tz: "utc".into(),
                }
            };
        }
        add(&mut conn, location!("US", "AZ", "US City", "us_city"), &weather_dir).unwrap();
        add(&mut conn, location!("US", "OR", "US City", "us_city_copy"), &weather_dir).unwrap();
        add(&mut conn, location!("CA", "BC", "CA City", "ca_city"), &weather_dir).unwrap();
        add(&mut conn, location!("CA", "ON", "CA City", "ca_city_copy"), &weather_dir).unwrap();
        let locations = get(&conn, None).unwrap();
        assert_eq!(locations.len(), 4);
        let locations = get(&conn, Some(vec![LocationFilter::alias("*s_c*")])).unwrap();
        assert_eq!(locations.len(), 2);
        let locations = get(&conn, Some(vec![LocationFilter::city("* City")])).unwrap();
        assert_eq!(locations.len(), 4);
        let locations = get(&conn, Some(vec![LocationFilter::city("* city").with_country("us")])).unwrap();
        assert_eq!(locations.len(), 2);
        let locations = get(
            &conn,
            Some(vec![
                LocationFilter::alias("us_city").with_region("az").with_country("us"),
                LocationFilter::city("ca city").with_region("bc").with_country("ca"),
            ]),
        )
        .unwrap();
        assert_eq!(locations.len(), 2);
        let locations = get(&conn, Some(vec![LocationFilter::country("us"), LocationFilter::country("CA")])).unwrap();
        assert_eq!(locations.len(), 4);
    }

    #[test]
    fn add_delete() {
        let fixture = init_fixture(false);
        let weather_dir = WeatherDir::from(&fixture);
        let mut conn = weather::db_conn!(weather_dir).unwrap();

        // verify the initial state
        assert!(fs_lib::get_locations(&weather_dir, None).unwrap().is_empty());
        assert!(get(&conn, None).unwrap().is_empty());

        // add a location and verify the results
        let alias = "foothills";
        let added_location = Location {
            country_name: "United States".to_string(),
            country_code: "US".to_string(),
            region_name: "Arizona".to_string(),
            region_code: "AZ".to_string(),
            city_name: "Fortuna Foothills".to_string(),
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
            country_name: "".to_string(),
            country_code: "".to_string(),
            region_name: "".to_string(),
            region_code: "".to_string(),
            city_name: "Yuma".to_string(),
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
        assert_eq!(updated_fs_location.city_name, updated_location.city_name);
        assert_eq!(updated_fs_location.country_name, added_location.country_name);
        assert_eq!(updated_fs_location.country_code, added_location.country_code);
        assert_eq!(updated_fs_location.region_name, added_location.region_name);
        assert_eq!(updated_fs_location.region_code, added_location.region_code);
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
        let south_filter = || Some(vec![LocationFilter::alias(south)]);

        // get south from the locations document
        let weather_dir = WeatherDir::from(&fixture);
        let fs_location = fs_lib::get_locations(&weather_dir, south_filter()).unwrap().remove(0);

        // add the location to the database
        let mut conn = weather::db_conn!(weather_dir).unwrap();
        let tx = create_tx!(conn, "failed getting add transaction").unwrap();
        add_db(&tx, fs_location.clone()).unwrap();
        commit_tx!(tx, "failed adding location to test database").unwrap();

        // get the location from the db and make sure it's what you expect
        let db_locations = get(&conn, south_filter()).unwrap();
        assert_eq!(db_locations.len(), 1);
        assert_locations!(fs_location, db_locations[0]);

        // don't update a location with the same properties
        let tx = create_tx!(conn, "failed getting update transaction").unwrap();
        assert!(!update_db(&tx, &fs_location).unwrap());

        // make the changes
        let update = Location {
            country_name: "Country".to_string(),
            country_code: "CO".to_string(),
            region_name: "Region".to_string(),
            region_code: "RN".to_string(),
            city_name: "city".to_string(),
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
            country_name: "Country Name".to_string(),
            country_code: "CN".to_string(),
            region_name: "Xerces".to_string(),
            region_code: "XX".to_string(),
            city_name: "Some City".to_string(),
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
        assert_eq!(partial_update.country_name, partial.country_name);
        assert_eq!(partial_update.country_code, partial.country_code);
        assert_eq!(partial_update.region_name, partial.region_name);
        assert_eq!(partial_update.region_code, partial.region_code);
        assert_eq!(partial_update.city_name, partial.city_name);
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
        let mut conn = weather::db_conn!(weather_dir).unwrap();
        load(&mut conn, &weather_dir).unwrap();
        let db_locations = get(&conn, None).unwrap();

        // verify the locations
        let fs_locations = fs_lib::get_locations(&weather_dir, None).unwrap();
        assert_eq!(db_locations.len(), fs_locations.len());
        for (lhs, rhs) in db_locations.iter().zip(fs_locations.iter()) {
            assert_locations!(lhs, rhs);
        }

        // verify the helpers that get location row ids
        let id_aliases = id_aliases(&conn).unwrap();
        assert_eq!(id_aliases.len(), db_locations.len());
        for (id, alias) in id_aliases {
            assert_eq!(id, location_id(&conn, &alias).unwrap());
        }
    }
}
