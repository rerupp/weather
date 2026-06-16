//! The metadata table definition.
//!
use super::{create_insert_sql, DatesTbl, TblSqlBuilder};

pub const SCHEMA: &'static str = r"
CREATE TABLE IF NOT EXISTS metadata
(
    id                INTEGER PRIMARY KEY,
    did               INTEGER NOT NULL,
    uncompressed_size INTEGER,
    compressed_size   INTEGER,
    data_size         INTEGER,
    -- backlink to the associated date
    FOREIGN KEY (did) REFERENCES dates (id)
);
CREATE INDEX IF NOT EXISTS idx_fs_metadata_did on metadata (did);
";

#[derive(Debug)]
pub enum MetadataTbl {
    // the enum must follow the schema column order
    Id,
    Did,
    UncompressedSize,
    CompressedSize,
    DataSize,
}
impl MetadataTbl {
    // the array MUST be in enum order to stay in sync
    const COLUMN_PARAM: [(&str, &str); 5] = [
        ("id", ":id"),
        ("did", ":did"),
        ("uncompressed_size", ":uncompressed_size"),
        ("compressed_size", ":compressed_size"),
        ("data_size", ":data_size"),
    ];

    // The schema name for the table.
    pub const TABLE: &str = "metadata";

    /// Generate the SQL fragment '`table AS alias`'.
    ///
    /// # Arguments
    ///
    /// * `alias` is the table alias name.
    ///
    pub fn table_as(alias: impl ToString) -> String {
        format!("{} AS {}", Self::TABLE, alias.to_string())
    }

    /// Generate the SQL fragment '`metadata AS m ON d.id=m.did`'.
    ///
    /// # Arguments
    ///
    /// * `m` is the metadata table alias name.
    /// * `d` is the dates table alias name.
    ///
    pub fn alias_join_dates(m: &str, d: &str) -> String {
        format!("{} ON {}={}", Self::table_as(m), DatesTbl::Id.alias_column(d), Self::Did.alias_column(m))
    }

    /// Get the SQL that will insert a row into the table.
    ///
    pub fn insert_sql() -> String {
        create_insert_sql(Self::TABLE, &Self::COLUMN_PARAM)
    }
}
impl TblSqlBuilder for MetadataTbl {
    /// The column names.
    ///
    fn column(&self) -> &'static str {
        match self {
            Self::Id => Self::COLUMN_PARAM[Self::Id as usize].0,
            Self::Did => Self::COLUMN_PARAM[Self::Did as usize].0,
            Self::UncompressedSize => Self::COLUMN_PARAM[Self::UncompressedSize as usize].0,
            Self::CompressedSize => Self::COLUMN_PARAM[Self::CompressedSize as usize].0,
            Self::DataSize => Self::COLUMN_PARAM[Self::DataSize as usize].0,
        }
    }

    /// The column parameter names.
    ///
    fn param(&self) -> &'static str {
        match self {
            Self::Id => Self::COLUMN_PARAM[Self::Id as usize].1,
            Self::Did => Self::COLUMN_PARAM[Self::Did as usize].1,
            Self::UncompressedSize => Self::COLUMN_PARAM[Self::UncompressedSize as usize].1,
            Self::CompressedSize => Self::COLUMN_PARAM[Self::CompressedSize as usize].1,
            Self::DataSize => Self::COLUMN_PARAM[Self::DataSize as usize].1,
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
        entities::Location,
    };
    use chrono::NaiveDate;
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

        // add a metadata row
        let uncompressed_size = 321;
        let compressed_size = 123;
        let data_size = 456;
        let insert_sql = MetadataTbl::insert_sql();
        let mut stmt = prepare_sql!(conn, &insert_sql, "failed to prepare metadata insert SQL").unwrap();
        let params = [
            named_param!(MetadataTbl::Did, did),
            named_param!(MetadataTbl::UncompressedSize, uncompressed_size),
            named_param!(MetadataTbl::CompressedSize, compressed_size),
            named_param!(MetadataTbl::DataSize, data_size),
        ];
        execute_sql!(stmt, &params, "failed to insert metadata").unwrap();

        // verify the row contents
        let query_sql = sql::Select::new()
            .select(MetadataTbl::Did.column())
            .select(MetadataTbl::UncompressedSize.column())
            .select(MetadataTbl::CompressedSize.column())
            .select(MetadataTbl::DataSize.column())
            .from(MetadataTbl::TABLE)
            .to_string();
        let mut stmt = prepare_sql!(conn, &query_sql, "failed to prepare metadata row query").unwrap();
        stmt.query_one([], |row| {
            assert_eq!(row.get::<_, i64>(MetadataTbl::Did.column()).unwrap(), did);
            assert_eq!(row.get::<_, i64>(MetadataTbl::UncompressedSize.column()).unwrap(), uncompressed_size);
            assert_eq!(row.get::<_, i64>(MetadataTbl::CompressedSize.column()).unwrap(), compressed_size);
            assert_eq!(row.get::<_, i64>(MetadataTbl::DataSize.column()).unwrap(), data_size);
            Ok(())
        })
        .unwrap();
    }
}
