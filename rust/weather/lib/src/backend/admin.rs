//! The administration commands are scoped to this module.
use super::{db, filesys};
use crate::backend::filesys::WeatherDir;
use std::rc::Rc;

/// Re-export the administration APIs here allowing the backend modules to be private.
pub(crate) use crate::backend::{db::admin::DbAdmin, filesys::admin::FsAdmin};

/// Create the database [DbAdmin] administration API.
///
/// # Arguments
///
/// * `weather_dir` is the weather data directory.
///
pub(crate) fn create_db_admin(weather_dir: Rc<WeatherDir>) -> Box<dyn DbAdmin> {
    Box::new(db::admin::create_db_admin(weather_dir))
}
/// Create the filesystem [FsAdmin] administration API.
///
/// # Arguments
///
/// * `weather_dir` is the weather data directory.
///
pub(crate) fn create_fs_admin(weather_dir: Rc<WeatherDir>) -> FsAdmin {
    filesys::admin::create_fs_admin(weather_dir)
}
