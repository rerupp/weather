/// The US Cities administration API.
/// 
use crate::{
    backend::db::sqlite::{commit_tx, create_tx, err, execute_sql, prepare_sql, query_rows, SqlResult},
    entities::State,
    LogElapsedTime
};
use csv::{Reader, StringRecord};
use rusqlite::{named_params, Connection, Row, Statement, Transaction, };
use sql_query_builder as sql;
use std::path::PathBuf;

/// Initialize the US Cities database schema.
///
/// # Arguments
///
/// * `conn` is the database connection that will be used.
///
pub fn init_schema(conn: &Connection) -> crate::Result<()> {
    let schema_sql = include_str!("schema.sql");
    if let Err(error) = conn.execute_batch(schema_sql) {
        err!("failed to initialize US Cities database schema: {:?}", error)?;
    }
    Ok(())
}

/// Load the US Cities database.
///
/// # Arguments
///
/// * `conn` is the database connection that will be used.
/// * `path` is the US Cities source data.
///
pub fn load_db(conn: &mut Connection, path: PathBuf) -> crate::Result<usize> {
    crate::log_elapsed_time!("load_db");
    let cities = load_cities_source(&path)?;
    let tx = create_tx!(conn, "failed to create US Cities load transaction")?;
    let states = insert_states(&tx, &cities)?;
    let insert_timer = LogElapsedTime::new("CityWriter", Some(log::Level::Debug));
    let mut writer = CityWriter::new(&tx, states)?;
    for city in cities {
        writer.add_city(city)?;
    }
    drop(insert_timer);
    let count = writer.count;
    drop(writer);
    commit_tx!(tx, "failed to commit US Cities load")?;
    Ok(count)
}

/// The metadata mined from the US Cities CSV file.
#[derive(Debug)]
struct City {
    /// The name of the city.
    name: String,

    /// The city full state name.
    state: String,

    /// The city two-letter state name abbreviation.
    state_id: String,

    /// The city latitude.
    latitude: String,

    /// The city longitude.
    longitude: String,

    /// The city timezone.
    timezone: String,

    /// The zip codes associated with the city.
    zip_codes: String,
}
/// Create the city metadata from a CSV record.
impl From<&StringRecord> for City {
    fn from(record: &StringRecord) -> Self {
        City {
            name: record.get(1).map_or(Default::default(), |v| v.trim().to_string()),
            state: record.get(3).map_or(Default::default(), |v| v.trim().to_string()),
            state_id: record.get(2).map_or(Default::default(), |v| v.trim().to_string()),
            latitude: record.get(6).map_or(Default::default(), |v| v.trim().to_string()),
            longitude: record.get(7).map_or(Default::default(), |v| v.trim().to_string()),
            timezone: record.get(13).map_or(Default::default(), |v| v.trim().to_string()),
            zip_codes: record.get(15).map_or(Default::default(), |v| v.trim().to_string()),
        }
    }
}
/// Create the state metadata from city metadata.
impl From<&City> for State {
    fn from(city: &City) -> Self {
        Self { name: city.state.clone(), state_id: city.state_id.clone() }
    }
}

/// Get the collection of US Cities from the source file.
///
/// # Arguments
///
/// * `path` is the US Cities source file.
///
fn load_cities_source(path: &PathBuf) -> crate::Result<Vec<City>> {
    crate::log_elapsed_time!("load_source");
    if !path.exists() {
        err!("source file '{}' was not found.", path.display())
    } else if !path.is_file() {
        err!("'{}' is not a file.", path.display())
    } else {
        match Reader::from_path(&path) {
            Err(error) => err!("error getting CSV reader for '{}': {:?}", path.display(), error),
            Ok(reader) => {
                let mut cities = Vec::new();
                for next_result in reader.into_records() {
                    match next_result {
                        Err(error) => err!("error reading CSV record ({:?}).", error)?,
                        Ok(record) => {
                            let city = City::from(&record);
                            if city.name.is_empty() {
                                eprintln!("city name is empty: {:?}", record);
                            } else if city.state.is_empty() {
                                eprintln!("city state is empty: {:?}", record);
                            } else {
                                cities.push(city);
                            }
                        },
                    }
                }
                Ok(cities)
            }
        }
    }
}

/// The metadata associated with a states row.
struct StatesRow {
    /// The row primary id.
    id: i64,

    /// The state metadata.
    state: State,
}

/// Add state metadata data to the database capturing the primary id for each row.
///
/// Arguments
///
/// * `tx` is the database transaction that will be used.
/// * `cities` contains the state information to add.
///
fn insert_states(tx: &Transaction, cities: &Vec<City>) -> crate::Result<Vec<StatesRow>> {
    crate::log_elapsed_time!("insert_states");
    // collect the state information
    let mut rows: Vec<StatesRow> = vec![];
    cities
        .iter()
        .for_each(|city| {
            if !rows.iter().any(|row| &row.state.name == &city.state && &row.state.state_id == &city.state_id) {
                rows.push(StatesRow{ id: 0, state: State::from(city) });
            }
        });
    rows.sort_unstable_by(|lhs, rhs| {
        match lhs.state.name.cmp(&rhs.state.name) {
            std::cmp::Ordering::Equal => lhs.state.state_id.cmp(&rhs.state.state_id),
            ordering => ordering,
        }
    });

    // set the primary id for each row
    rows.iter_mut().zip(1i64..).for_each(|(row, id)| row.id = id);

    let mut insert = sql::Insert::new().insert_into("states(id, name, state_id)");
    for row in rows.iter() {
        insert = insert.values(&format!("({}, '{}', '{}')", row.id, row.state.name, row.state.state_id))
    }
    let mut insert_stmt = prepare_sql!(tx, &insert.to_string(), "failed to prepare states INSERT SQL")?;
    execute_sql!(insert_stmt, [], "failed to insert states")?;
    Ok(rows)
}

/// The city metadata database writer.
struct CityWriter<'t> {
    /// The database transaction that will be used.
    tx: &'t Transaction<'t>,

    /// The prepared cities table insert statement.
    city_insert_stmt: Statement<'t>,

    /// The number of rows the writer has added.
    count: usize,

    /// The states table rows.
    states: Vec<StatesRow>,
}
impl<'t> CityWriter<'t> {
    /// Create a new US Cities database writer.
    ///
    /// # Arguments
    ///
    /// * `tx` is the database transaction that will be used.
    /// * `states` holds the contents of the states table.
    ///
    fn new(tx: &'t Transaction, states: Vec<StatesRow>) -> crate::Result<Self> {
        const CITY_SQL: &str = r#"
        INSERT INTO cities (city, states_id, latitude, longitude, timezone)
            VALUES (:city, :states_id, :lat, :long, :tz)
        "#;
        let city_insert_stmt = prepare_sql!(tx, CITY_SQL, "failed to prepare INSERT City SQL")?;
        Ok(Self { tx, city_insert_stmt, count: 0, states })
    }

    /// Add city metadata to the `cities` table.
    ///
    /// # Arguments
    ///
    /// * `city` is the metadata that will be added.
    ///
    fn add_city(&mut self, city: City) -> crate::Result<()> {
        // find the city states row.
        let states_id = self.states
            .iter()
            .find(|row| &row.state.name == &city.state )
            .map(|row| row.id);
        // warn someone if the state was not found.
        if states_id.is_none() {
            eprintln!("did not find city state name in states table: {:?}", city);
        } else {
            let params = named_params! {
                ":city": &city.name,
                ":states_id": states_id.unwrap(),
                ":lat": &city.latitude,
                ":long": &city.longitude,
                ":tz": &city.timezone
            };
            execute_sql!(self.city_insert_stmt, params, "failed to insert US Cities record into DB")?;
            self.insert_zip_codes(self.tx.last_insert_rowid(), city)?;
            self.count += 1;
        }
        Ok(())
    }

    /// Insert the zip codes associate with the city.
    ///
    /// # Arguments
    ///
    /// * `city_id` is the city database primary id.
    /// * `city` is the city metadata.
    ///
    fn insert_zip_codes(&self, city_id: i64, city: City) -> crate::Result<()> {
        if !city.zip_codes.is_empty() {
            let mut insert = sql::Insert::new().insert_or("IGNORE into zip_codes(zip_code)");
            let zip_codes = city.zip_codes.split_whitespace().collect::<Vec<_>>();
            for zip_code in &zip_codes {
                insert = insert.values(&format!("('{zip_code}')"));
            }
            let mut stmt = prepare_sql!(self.tx, &insert.to_string(), "failed to prepare INSERT Zip code SQL")?;
            execute_sql!(stmt, [], "failed to INSERT zip codes")?;
            self.insert_city_zip_codes(city_id, zip_codes)?;
        }
        Ok(())
    }

    /// Add the city zip codes to the join table.
    ///
    /// # Arguments
    ///
    /// * `city_id` is the city database primary id.
    /// * `zip_codes` are the zip codes associated with the city.
    ///
    fn insert_city_zip_codes(&self, city_id: i64, zip_codes: Vec<&str>) -> crate::Result<()> {
        // query the zip code row ids
        let zip_codes = zip_codes.into_iter().map(|s| format!("'{s}'")).collect::<Vec<_>>().join(",");
        let query = format!("SELECT id FROM zip_codes WHERE zip_code IN ({zip_codes})");
        let mut query_stmt = prepare_sql!(self.tx, &query, "failed to prepare SELECT Zip code id SQL")?;
        let mut rows = query_rows!(query_stmt, [], "failed to get Zip code IDs")?;

        // create the insert statement
        let mut insert = sql::Insert::new().insert_into("city_zip_codes(cities_id, zip_codes_id)");
        loop {
            match rows.next() {
                Err(error) => err!("failed to get next Zip code ID: {:?}", error)?,
                // the caller guarantees there are zip codes to add
                Ok(None) => break,
                Ok(Some(row)) => {
                    #[inline]
                    fn next_zip_code(row_: &Row) -> SqlResult<i64> {
                        row_.get(0)
                    }
                    match next_zip_code(row) {
                        Err(error) => err!("failed to get Zip code ID: {:?}", error)?,
                        Ok(zip_code_id) => insert = insert.values(&format!("('{city_id}', '{zip_code_id}')")),
                    }
                }
            }
        }
        let mut insert_stmt = prepare_sql!(self.tx, &insert.to_string(), "failed to prepare INSERT Zip code SQL")?;
        execute_sql!(insert_stmt, [], "failed to INSERT Zip codes")
    }
}

pub fn state_metrics(conn: &Connection) -> crate::Result<Vec<(String, usize)>> {
    const QUERY: &str = "
        SELECT states.state_id AS state_id, COUNT(*)
        FROM cities
        INNER JOIN states on cities.states_id=states.id
        GROUP BY state_id
        ORDER BY state_id
    ";
    let mut stmt = prepare_sql!(conn, QUERY, "failed to prepare state metrics query")?;
    let mut rows = query_rows!(stmt, [], "failed to query state metrics")?;

    let mut state_info: Vec<(String, usize)> = Vec::with_capacity(52);
    loop {
        match rows.next() {
            Err(error) => err!("failed to get next metrics row: {:?}", error)?,
            Ok(None) => break,
            Ok(Some(row)) => {
                #[inline]
                fn next_state_count(row_: &Row) -> SqlResult<(String, usize)> {
                    Ok((row_.get(0)?, row_.get(1)?))
                }
                match next_state_count(row) {
                    Ok(state_count) => state_info.push(state_count),
                    Err(error) => err!("failed to get state metrics: {:?}", error)?,
                }
            }
        }
    }
    Ok(state_info)
}
