//! This module manages weather data history in the database.
mod query;
pub use query::history_dates;

pub mod persistence;

use crate::{
    admin::entities::{DbHistoryProblemDetails, DbHistoryProblems},
    backend::{
        db::sqlite::{
            commit_tx, create_tx,
            tables::weather::{DatesTbl, HistoryTbl, MetadataTbl},
            weather::locations,
        },
        filesys::{fs_lib, FilesysMetadata, WeatherDir},
    },
    entities::{DailyHistories, DateRange, History, HistorySummary, Location, LocationFilter},
};
use chrono::NaiveDate;
use rusqlite::{Connection, Transaction};

/// Create an error from history specific error message.
macro_rules! err {
    ($($arg:tt)*) => {
        Err(crate::Error::from(format!("history {}", format!($($arg)*))))
    };
}

/// Add history to the filesystem and database storage.
///
/// # Arguments
///
/// * `conn` is the database connection used to add the history data.
/// * `weather_dir` is the historical weather data directory.
/// * `daily_histories` has the new historical weather data.
///
pub fn add(
    conn: &mut Connection,
    weather_dir: &WeatherDir,
    mut daily_histories: DailyHistories,
) -> crate::Result<usize> {
    // make sure the database knows about the location
    let lid = locations::location_id(conn, &daily_histories.location.alias)?;

    // the history archive will make sure there are no duplicates added and issue log warnings
    let fs_history_metadata = fs_lib::daily_history::add(weather_dir, &mut daily_histories)?;
    if fs_history_metadata.is_empty() {
        return Ok(0);
    }

    // check if the filesystem purged some of the daily histories
    if fs_history_metadata.len() != daily_histories.histories.len() {
        use std::collections::HashSet;
        let include_history_filter = fs_history_metadata.iter().map(|history| history.date).collect::<HashSet<_>>();
        daily_histories.histories.retain(|history| include_history_filter.contains(&history.date));
    }

    // make sure the metadata and histories are in sync
    #[cfg(debug_assertions)]
    {
        assert_eq!(fs_history_metadata.len(), daily_histories.histories.len());
    }

    // combine the metadata and histories that will be inserted
    let new_metadata_history: Vec<(FilesysMetadata, History)> =
        fs_history_metadata.into_iter().zip(daily_histories.histories.into_iter()).collect();

    // add the histories
    let mut tx = create_tx!(conn, "failed to create insert transaction")?;
    let dates = InsertMgr::new().bulk_insert(&mut tx, lid, new_metadata_history)?;
    commit_tx!(tx, "failed to commit daily histories")?;
    Ok(dates.len())
}

/// The [InsertMgr] is used to add  weather history data into the database.
#[derive(Debug)]
pub struct InsertMgr {
    /// The SQL that inserts data into the dates table.
    dates_insert: String,
    /// The SQL that inserts data into the metadata table.
    metadata_insert: String,
    /// The SQL that inserts data into the history table.
    history_insert: String,
}
impl InsertMgr {
    /// Creates a new instance of the manager.
    pub fn new() -> Self {
        Self {
            dates_insert: DatesTbl::insert_sql(),
            metadata_insert: MetadataTbl::insert_sql(),
            history_insert: HistoryTbl::insert_sql(),
        }
    }

    /// Inserts a collection of [FilesysMetadata] and [History] into the database.
    ///
    /// # Arguments
    ///
    /// * `tx` is the transaction that will be used for the inserts.
    /// * `lid` is the location ROWID
    /// * `metadata_history` is the collection of filesystem metadata and associated weather history data.
    ///
    pub fn bulk_insert(
        &self,
        tx: &mut Transaction,
        lid: i64,
        metadata_history: Vec<(FilesysMetadata, History)>,
    ) -> crate::Result<Vec<NaiveDate>> {
        let mut dates = vec![];
        for (metadata, history) in metadata_history {
            dates.push(metadata.date);
            self.insert(tx, lid, metadata, history)?;
        }
        Ok(dates)
    }

    /// Add a locations filesystem metadata and history data into the database.
    ///
    /// # Arguments
    ///
    /// * `tx` is the transaction that will be used for the insert.
    /// * `lid` is the location ROWID.
    /// * `metadata` is the filesystem metadata that will be added.
    /// * `history` is the weather history data that will be added.
    ///
    pub fn insert(
        &self,
        tx: &mut Transaction,
        lid: i64,
        metadata: FilesysMetadata,
        history: History,
    ) -> crate::Result<()> {
        #[cfg(debug_assertions)]
        {
            assert_eq!(metadata.date, history.date);
        }
        let did = persistence::dates::insert(tx, lid, &self.dates_insert, history.date)?;
        persistence::metadata::insert(tx, did, &self.metadata_insert, &metadata)?;
        persistence::history::insert(tx, did, &self.history_insert, &history)?;
        Ok(())
    }
}

/// Get the daily weather data history for a location.
///
/// # Arguments
///
/// * `conn` is the database connection that will be used.
/// * `location` is whose history will be returned.
/// * `date_range` is the history dates to query.
///
pub fn get(conn: &mut Connection, location: Location, date_range: DateRange) -> crate::Result<DailyHistories> {
    let histories = query::get_history(&conn, &location.alias, date_range)?;
    Ok(DailyHistories { location, histories })
}

/// Get a summary of the weather history available for locations.
///
/// # Arguments
///
/// * `conn` is the database connection that will be used.
/// * `weather_dir` is the weather data history directory.
/// * `filters_opt` is used to optionally restrict what location summaries are returned.
///
pub fn summary(
    conn: &mut Connection,
    weather_dir: &WeatherDir,
    filters_opt: Option<Vec<LocationFilter>>,
) -> crate::Result<Vec<HistorySummary>> {
    crate::log_elapsed_time!("summary");
    let mut db_sizes = query::db_size(&conn)?;
    let mut fs_sizes = query::fs_size(&conn, weather_dir)?;
    let history_summaries = locations::get(&conn, filters_opt)?
        .into_iter()
        .map(|location| {
            let (days, db_metadata) = db_sizes.remove(&location.alias).expect("Did not find db metadata for location");
            let fs_metadata = fs_sizes.remove(&location.alias).expect("Did not find fs metadata for location");
            HistorySummary {
                location,
                days: days as u64,
                fs_history_summary: fs_metadata,
                db_history_summary: Some(db_metadata),
            }
        })
        .collect();
    Ok(history_summaries)
}

/// Reload a locations weather history for the *normalized* implementation of weather data.
///
/// # Argument
///
/// * `conn` is the database connection that will be used.
/// * `weather_dir` is the weather data directory.
/// * `alias` is the location that will be reloaded.
///
pub fn reload(conn: &mut Connection, weather_dir: &WeatherDir, alias: &str) -> crate::Result<()> {
    crate::log_elapsed_time!("reload");
    let lid = locations::location_id(conn, alias)?;
    let mut tx = create_tx!(conn, "failed to create reload transaction")?;
    delete(&mut tx, lid)?;
    let metadata_history = fs_lib::history_contents(weather_dir, alias)?.collect::<Vec<_>>();
    InsertMgr::new().bulk_insert(&mut tx, lid, metadata_history)?;
    commit_tx!(tx, "failed to commit reload for '{alias}'")
}

/// Delete the history for a specific location.
///
/// # Arguments
///
/// * `tx` is the transaction that will be used.
/// * `lid` is the database location id.
///
pub fn delete(tx: &mut Transaction, lid: i64) -> crate::Result<()> {
    crate::log_elapsed_time!("history delete");
    persistence::history::delete(tx, lid)?;
    persistence::metadata::delete(tx, lid)?;
    persistence::dates::delete(tx, lid)?;
    Ok(())
}

/// Check the history state in the database to the contents of the filesystem store.
///
/// # Arguments
///
/// * `conn` is the database connection that will be used.
/// * `weather_dir` is the weather history data directory.
///
pub fn check(conn: &mut Connection, weather_dir: &WeatherDir) -> crate::Result<Option<DbHistoryProblems>> {
    let fs_history_counts = fs_lib::history_counts::get(weather_dir, None)?;
    let mut history_problems = vec![];
    let mut detached_store = vec![];
    for summary in summary(conn, weather_dir, None)? {
        match fs_history_counts.iter().find(|(l, _)| &summary.location.alias == &l.alias) {
            None => detached_store.push(summary),
            Some((_, fs_count)) => {
                if summary.days as usize != *fs_count {
                    history_problems.push(DbHistoryProblemDetails {
                        location: summary.location,
                        db_histories: summary.days as usize,
                        fs_histories: *fs_count,
                    });
                }
            }
        }
    }
    match history_problems.len() > 0 || detached_store.len() > 0 {
        false => Ok(None),
        true => {
            let mut problems = DbHistoryProblems::default();
            if history_problems.len() > 0 {
                problems.history_problems.replace(history_problems);
            }
            if detached_store.len() > 0 {
                problems.detached_store.replace(detached_store);
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
            admin::{create_db_admin, DbAdmin},
            sqlite::{
                prepare_sql,
                tables::{named_param, weather::LocationsTbl, TblSqlBuilder},
                weather,
            },
        },
        testlib,
    };
    use chrono::NaiveDate;
    use sql_query_builder as sql;
    use std::path::PathBuf;

    #[test]
    fn delete_history() {
        // use the database test resources
        let fixture = testlib::TestFixture::create();
        fixture.copy_resources(&testlib::test_resources().join("db"));
        let fixture_path = PathBuf::from(&fixture);

        // initialize the database
        let db_admin = Box::new(create_db_admin(WeatherDir::new(fixture_path.clone()).unwrap()));
        db_admin.history_init(false).unwrap();
        db_admin.history_load(3).unwrap();

        // set up the test environment
        let weather_dir = WeatherDir::new(PathBuf::from(&fixture)).unwrap();
        let mut conn = weather::db_conn!(weather_dir).unwrap();
        let lid = locations::location_id(&conn, "south").unwrap();

        fn history_count(conn: &Connection, lid: i64) -> usize {
            let l = "l";
            let d = "d";
            let count_sql = sql::Select::new()
                .select("COUNT(*)")
                .from(&LocationsTbl::table_as(l))
                .inner_join(&DatesTbl::alias_join_locations_as(d, l))
                .where_clause(&LocationsTbl::Id.alias_where_param(l))
                .to_string();
            let mut stmt = prepare_sql!(conn, &count_sql, "error preparing dates count").unwrap();
            let count = stmt.query_one(&[named_param!(LocationsTbl::Id, lid)], |row| row.get::<_, i64>(0)).unwrap();
            count as usize
        }
        assert_eq!(history_count(&conn, lid), 29);
        let mut tx = create_tx!(conn, "failed to create delete TX").unwrap();
        delete(&mut tx, lid).unwrap();
        commit_tx!(tx, "failed to commit delete TX").unwrap();
        assert_eq!(history_count(&conn, lid), 0);
    }

    #[test]
    fn add_history() {
        // use the database test resources
        let fixture = testlib::TestFixture::create();
        fixture.copy_resources(&testlib::test_resources().join("db"));
        let fixture_path = PathBuf::from(&fixture);

        // initialize the database
        let db_admin = Box::new(create_db_admin(WeatherDir::new(fixture_path.clone()).unwrap()));
        db_admin.history_init(false).unwrap();
        db_admin.history_load(3).unwrap();

        // set up the test environment
        let weather_dir = WeatherDir::new(PathBuf::from(&fixture)).unwrap();
        let mut conn = weather::db_conn!(weather_dir).unwrap();

        // verify there are no issues between the database and filesystem
        macro_rules! check {
            ($what:expr) => {
                if let Some(db_problems) = check(&mut conn, &weather_dir).unwrap() {
                    println!("{} problems: {:?}", $what, db_problems);
                    assert!(false);
                }
            };
        }
        check!("Init");

        // get the locations
        let mut locations = locations::get(&conn, None).unwrap();
        let south = locations.pop().unwrap();
        assert_eq!(south.alias, "south");
        let north = locations.pop().unwrap();
        assert_eq!(north.alias, "north");
        let between = locations.pop().unwrap();
        assert_eq!(between.alias, "between");

        // capture the history counts before adding any histories
        let before_history_counts = query::history_counts(&conn).unwrap();

        macro_rules! date {
            ($y:expr, $m:expr, $d:expr) => {
                NaiveDate::from_ymd_opt($y, $m, $d).unwrap()
            };
        }

        macro_rules! histories {
            ($location:expr, $start:expr, $end:expr) => {
                DateRange::new($start, $end)
                    .into_iter()
                    .map(|date| History { alias: $location.alias.clone(), date, ..Default::default() })
                    .collect::<Vec<_>>()
            };
        }

        // add histories that do not overlap
        let histories = histories!(north, date!(2025, 10, 1), date!(2025, 10, 31));
        let daily_histories = DailyHistories { location: north, histories };
        let add_count = add(&mut conn, &weather_dir, daily_histories).unwrap();
        assert_eq!(add_count, 31);
        check!("Add all");

        // capture the history counts after adding the histories
        let north_history_counts = query::history_counts(&conn).unwrap();
        assert_eq!(*north_history_counts.get("north").unwrap(), before_history_counts.get("north").unwrap() + 31);

        // add histories that partially overlap
        let histories = histories!(south, date!(2015, 4, 7), date!(2015, 4, 21));
        let daily_histories = DailyHistories { location: south, histories };
        let add_count = add(&mut conn, &weather_dir, daily_histories).unwrap();
        assert_eq!(add_count, 7);
        check!("Partial add");

        // verify the history counts after partially adding the histories
        let south_history_counts = query::history_counts(&conn).unwrap();
        assert_eq!(*south_history_counts.get("south").unwrap(), before_history_counts.get("south").unwrap() + 7);

        // add histories that all overlap
        let histories = histories!(between, date!(2015, 4, 1), date!(2015, 4, 14));
        let daily_histories = DailyHistories { location: between, histories };
        let add_count = add(&mut conn, &weather_dir, daily_histories).unwrap();
        assert_eq!(add_count, 0);
        check!("No add");

        // verify the history counts after trying to add
        let between_history_counts = query::history_counts(&conn).unwrap();
        assert_eq!(between_history_counts.get("between"), before_history_counts.get("between"));
    }
}
