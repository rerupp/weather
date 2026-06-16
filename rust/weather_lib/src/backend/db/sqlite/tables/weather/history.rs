//! The history table definition.
use super::{create_insert_sql, dates::DatesTbl, TblSqlBuilder};

pub const SCHEMA: &'static str = r"
CREATE TABLE IF NOT EXISTS history
(
    id            INTEGER PRIMARY KEY,
    did           INTEGER NOT NULL,
    temp_high     REAL,
    temp_low      REAL,
    temp_mean     REAL,
    dew_point     REAL,
    humidity      REAL,
    sunrise_t     INTEGER,
    sunset_t      INTEGER,
    cloud_cover   REAL,
    moon_phase    REAL,
    uv_index      REAL,
    wind_speed    REAL,
    wind_gust     REAL,
    wind_dir      INTEGER,
    visibility    REAL,
    pressure      REAL,
    precip        REAL,
    precip_prob   REAL,
    precip_type   TEXT,
    description   TEXT,
    -- backlink to the associated date
    FOREIGN KEY (did) REFERENCES dates (id)
);
CREATE INDEX IF NOT EXISTS idx_history_did on history (did);
";

#[derive(Debug)]
pub enum HistoryTbl {
    // the enum must follow the schema column order
    Id,
    Did,
    TempHigh,
    TempLow,
    TempMean,
    DewPoint,
    Humidity,
    Sunrise,
    Sunset,
    CloudCover,
    MoonPhase,
    UvIndex,
    WindSpeed,
    WindGust,
    WindDir,
    Visibility,
    Pressure,
    Precip,
    PrecipProb,
    PrecipType,
    Description,
}
impl HistoryTbl {
    // the array MUST be in enum order to stay in sync
    const COLUMN_PARAM: [(&str, &str); 21] = [
        ("id", ":id"),
        ("did", ":did"),
        ("temp_high", ":temp_high"),
        ("temp_low", ":temp_low"),
        ("temp_mean", ":temp_mean"),
        ("dew_point", ":dew_point"),
        ("humidity", ":humidity"),
        ("sunrise_t", ":sunrise_t"),
        ("sunset_t", ":sunset_t"),
        ("cloud_cover", ":cloud_cover"),
        ("moon_phase", ":moon_phase"),
        ("uv_index", ":uv_index"),
        ("wind_speed", ":wind_speed"),
        ("wind_gust", ":wind_gust"),
        ("wind_dir", ":wind_dir"),
        ("visibility", ":visibility"),
        ("pressure", ":pressure"),
        ("precip", ":precip"),
        ("precip_prob", ":precip_prob"),
        ("precip_type", ":precip_type"),
        ("description", ":description"),
    ];

    /// The schema name for the table.
    pub const TABLE: &str = "history";

    /// Generate the SQL fragment '`table AS alias`'.
    ///
    /// # Arguments
    ///
    /// * `alias` is the table alias name.
    ///
    pub fn table_as(alias: &str) -> String {
        format!("{} AS {}", Self::TABLE, alias.to_string())
    }

    /// Get the SQL that will insert a row into the table.
    ///
    pub fn insert_sql() -> String {
        create_insert_sql(Self::TABLE, &Self::COLUMN_PARAM)
    }

    /// Generate the SQL fragment '`history AS h ON d.id=h.did`'.
    ///
    /// # Arguments
    ///
    /// * `h` is the history table alias.
    /// * `d` is the dates table alias.
    ///
    pub fn alias_join_dates_as(h: &str, d: &str) -> String {
        format!("{} ON {}={}", HistoryTbl::table_as(h), DatesTbl::Id.alias_column(d), HistoryTbl::Did.alias_column(h))
    }
}
impl TblSqlBuilder for HistoryTbl {
    /// The column names.
    ///
    fn column(&self) -> &'static str {
        match self {
            Self::Id => Self::COLUMN_PARAM[Self::Id as usize].0,
            Self::Did => Self::COLUMN_PARAM[Self::Did as usize].0,
            Self::TempHigh => Self::COLUMN_PARAM[Self::TempHigh as usize].0,
            Self::TempLow => Self::COLUMN_PARAM[Self::TempLow as usize].0,
            Self::TempMean => Self::COLUMN_PARAM[Self::TempMean as usize].0,
            Self::DewPoint => Self::COLUMN_PARAM[Self::DewPoint as usize].0,
            Self::Humidity => Self::COLUMN_PARAM[Self::Humidity as usize].0,
            Self::Sunrise => Self::COLUMN_PARAM[Self::Sunrise as usize].0,
            Self::Sunset => Self::COLUMN_PARAM[Self::Sunset as usize].0,
            Self::CloudCover => Self::COLUMN_PARAM[Self::CloudCover as usize].0,
            Self::MoonPhase => Self::COLUMN_PARAM[Self::MoonPhase as usize].0,
            Self::UvIndex => Self::COLUMN_PARAM[Self::UvIndex as usize].0,
            Self::WindSpeed => Self::COLUMN_PARAM[Self::WindSpeed as usize].0,
            Self::WindGust => Self::COLUMN_PARAM[Self::WindGust as usize].0,
            Self::WindDir => Self::COLUMN_PARAM[Self::WindDir as usize].0,
            Self::Visibility => Self::COLUMN_PARAM[Self::Visibility as usize].0,
            Self::Pressure => Self::COLUMN_PARAM[Self::Pressure as usize].0,
            Self::Precip => Self::COLUMN_PARAM[Self::Precip as usize].0,
            Self::PrecipProb => Self::COLUMN_PARAM[Self::PrecipProb as usize].0,
            Self::PrecipType => Self::COLUMN_PARAM[Self::PrecipType as usize].0,
            Self::Description => Self::COLUMN_PARAM[Self::Description as usize].0,
        }
    }

    /// The column parameter names.
    ///
    fn param(&self) -> &'static str {
        match self {
            Self::Id => Self::COLUMN_PARAM[Self::Id as usize].1,
            Self::Did => Self::COLUMN_PARAM[Self::Did as usize].1,
            Self::TempHigh => Self::COLUMN_PARAM[Self::TempHigh as usize].1,
            Self::TempLow => Self::COLUMN_PARAM[Self::TempLow as usize].1,
            Self::TempMean => Self::COLUMN_PARAM[Self::TempMean as usize].1,
            Self::DewPoint => Self::COLUMN_PARAM[Self::DewPoint as usize].1,
            Self::Humidity => Self::COLUMN_PARAM[Self::Humidity as usize].1,
            Self::Sunrise => Self::COLUMN_PARAM[Self::Sunrise as usize].1,
            Self::Sunset => Self::COLUMN_PARAM[Self::Sunset as usize].1,
            Self::CloudCover => Self::COLUMN_PARAM[Self::CloudCover as usize].1,
            Self::MoonPhase => Self::COLUMN_PARAM[Self::MoonPhase as usize].1,
            Self::UvIndex => Self::COLUMN_PARAM[Self::UvIndex as usize].1,
            Self::WindSpeed => Self::COLUMN_PARAM[Self::WindSpeed as usize].1,
            Self::WindGust => Self::COLUMN_PARAM[Self::WindGust as usize].1,
            Self::WindDir => Self::COLUMN_PARAM[Self::WindDir as usize].1,
            Self::Visibility => Self::COLUMN_PARAM[Self::Visibility as usize].1,
            Self::Pressure => Self::COLUMN_PARAM[Self::Pressure as usize].1,
            Self::Precip => Self::COLUMN_PARAM[Self::Precip as usize].1,
            Self::PrecipProb => Self::COLUMN_PARAM[Self::PrecipProb as usize].1,
            Self::PrecipType => Self::COLUMN_PARAM[Self::PrecipType as usize].1,
            Self::Description => Self::COLUMN_PARAM[Self::Description as usize].1,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{
        backend::db::sqlite::{
            db_connection, err, execute_sql, prepare_sql,
            tables::{named_param, weather},
        },
        entities::{History, Location},
    };
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
    use sql_query_builder as sql;

    #[test]
    fn row_insert() {
        // use an in-memory database
        let mut conn = db_connection(None).unwrap();
        weather::initialize_schema(&conn).unwrap();

        // add the associated content
        let location = Location { alias: "alias".to_string(), ..Default::default() };
        let lid = weather::locations::tests::insert_location(&mut conn, location);
        let date = NaiveDate::from_ymd_opt(2026, 6, 9).unwrap();
        let did = weather::dates::tests::insert_date(&mut conn, lid, date);

        // add a history row
        let history = History {
            temperature_high: Some(90.0),
            temperature_low: Some(60.0),
            temperature_mean: Some(75.0),
            dew_point: Some(45.0),
            humidity: Some(20.0),
            precipitation_chance: Some(0.1),
            precipitation_type: Some("rain".into()),
            precipitation_amount: Some(0.05),
            wind_speed: Some(5.0),
            wind_gust: Some(10.0),
            wind_direction: Some(270),
            cloud_cover: Some(0.25),
            pressure: Some(0.65),
            uv_index: Some(6.25),
            sunrise: Some(NaiveDateTime::new(date, NaiveTime::from_hms_opt(5, 30, 0).unwrap())),
            sunset: Some(NaiveDateTime::new(date, NaiveTime::from_hms_opt(8, 45, 0).unwrap())),
            moon_phase: Some(0.5),
            visibility: Some(100.0),
            description: Some("weather condition description".into()),
            ..Default::default()
        };
        let insert_sql = HistoryTbl::insert_sql();
        let mut stmt = prepare_sql!(conn, &insert_sql, "failed to prepare history insert SQL").unwrap();
        let params = [
            named_param!(HistoryTbl::Did, did),
            named_param!(HistoryTbl::TempHigh, history.temperature_high),
            named_param!(HistoryTbl::TempLow, history.temperature_low),
            named_param!(HistoryTbl::TempMean, history.temperature_mean),
            named_param!(HistoryTbl::DewPoint, history.dew_point),
            named_param!(HistoryTbl::Humidity, history.humidity),
            named_param!(HistoryTbl::PrecipProb, history.precipitation_chance),
            named_param!(HistoryTbl::PrecipType, history.precipitation_type),
            named_param!(HistoryTbl::Precip, history.precipitation_amount),
            named_param!(HistoryTbl::WindSpeed, history.wind_speed),
            named_param!(HistoryTbl::WindGust, history.wind_gust),
            named_param!(HistoryTbl::WindDir, history.wind_direction),
            named_param!(HistoryTbl::CloudCover, history.cloud_cover),
            named_param!(HistoryTbl::Pressure, history.pressure),
            named_param!(HistoryTbl::UvIndex, history.uv_index),
            named_param!(HistoryTbl::Sunrise, history.sunrise),
            named_param!(HistoryTbl::Sunset, history.sunset),
            named_param!(HistoryTbl::MoonPhase, history.moon_phase),
            named_param!(HistoryTbl::Visibility, history.visibility),
            named_param!(HistoryTbl::Description, history.description),
        ];
        execute_sql!(stmt, &params, "failed to insert history").unwrap();

        // verify the row contents
        let query_sql = sql::Select::new()
            .select(HistoryTbl::Did.column())
            .select(HistoryTbl::TempHigh.column())
            .select(HistoryTbl::TempLow.column())
            .select(HistoryTbl::TempMean.column())
            .select(HistoryTbl::DewPoint.column())
            .select(HistoryTbl::Humidity.column())
            .select(HistoryTbl::PrecipProb.column())
            .select(HistoryTbl::PrecipType.column())
            .select(HistoryTbl::Precip.column())
            .select(HistoryTbl::WindSpeed.column())
            .select(HistoryTbl::WindGust.column())
            .select(HistoryTbl::WindDir.column())
            .select(HistoryTbl::CloudCover.column())
            .select(HistoryTbl::Pressure.column())
            .select(HistoryTbl::UvIndex.column())
            .select(HistoryTbl::Sunrise.column())
            .select(HistoryTbl::Sunset.column())
            .select(HistoryTbl::MoonPhase.column())
            .select(HistoryTbl::Visibility.column())
            .select(HistoryTbl::Description.column())
            .from(HistoryTbl::TABLE)
            .to_string();
        let mut stmt = prepare_sql!(conn, &query_sql, "failed to prepare history row query").unwrap();
        stmt.query_one([], |row| {
            assert_eq!(row.get::<_, i64>(HistoryTbl::Did.column()).unwrap(), did);
            assert_eq!(row.get::<_, f64>(HistoryTbl::TempHigh.column()).unwrap(), history.temperature_high.unwrap());
            assert_eq!(row.get::<_, f64>(HistoryTbl::TempLow.column()).unwrap(), history.temperature_low.unwrap());
            assert_eq!(row.get::<_, f64>(HistoryTbl::TempMean.column()).unwrap(), history.temperature_mean.unwrap());
            assert_eq!(row.get::<_, f64>(HistoryTbl::DewPoint.column()).unwrap(), history.dew_point.unwrap());
            assert_eq!(row.get::<_, f64>(HistoryTbl::Humidity.column()).unwrap(), history.humidity.unwrap());
            assert_eq!(
                row.get::<_, String>(HistoryTbl::PrecipType.column()).unwrap(),
                history.precipitation_type.unwrap()
            );
            assert_eq!(row.get::<_, f64>(HistoryTbl::Precip.column()).unwrap(), history.precipitation_amount.unwrap());
            assert_eq!(row.get::<_, f64>(HistoryTbl::WindSpeed.column()).unwrap(), history.wind_speed.unwrap());
            assert_eq!(row.get::<_, f64>(HistoryTbl::WindGust.column()).unwrap(), history.wind_gust.unwrap());
            assert_eq!(row.get::<_, i64>(HistoryTbl::WindDir.column()).unwrap(), history.wind_direction.unwrap());
            assert_eq!(row.get::<_, f64>(HistoryTbl::CloudCover.column()).unwrap(), history.cloud_cover.unwrap());
            assert_eq!(row.get::<_, f64>(HistoryTbl::Pressure.column()).unwrap(), history.pressure.unwrap());
            assert_eq!(row.get::<_, f64>(HistoryTbl::UvIndex.column()).unwrap(), history.uv_index.unwrap());
            assert_eq!(row.get::<_, NaiveDateTime>(HistoryTbl::Sunrise.column()).unwrap(), history.sunrise.unwrap());
            assert_eq!(row.get::<_, NaiveDateTime>(HistoryTbl::Sunset.column()).unwrap(), history.sunset.unwrap());
            assert_eq!(row.get::<_, f64>(HistoryTbl::MoonPhase.column()).unwrap(), history.moon_phase.unwrap());
            assert_eq!(row.get::<_, f64>(HistoryTbl::Visibility.column()).unwrap(), history.visibility.unwrap());
            assert_eq!(row.get::<_, String>(HistoryTbl::Description.column()).unwrap(), history.description.unwrap());
            Ok(())
        })
        .unwrap();
    }
}
