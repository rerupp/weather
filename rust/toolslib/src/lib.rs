//! # A collection of utilities used by other crates in the playground.
//!
//! The intent of this library is to consolidate common code so I don't keep
//! duplicating it in each of the modules.
// #![feature(log_syntax)]
// #![feature(trace_macros)]
use std::{result, fmt::{Display, Formatter, Result as FmtResult}};
pub mod date_time;
pub mod fmt;
pub mod logs;
pub mod stopwatch;
pub mod text;
pub mod report;

/// The tools library result.
type Result<T> = result::Result<T, Error>;

/// The tools library Error that can be captured outside the module.
///
/// Currently it contains only a String but can be extended to an enum later on.
#[derive(Debug)]
pub struct Error(String);
/// Include the `ToString` trait for the [`Error`].
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}
/// Create a text error from a String.
impl From<String> for Error {
    fn from(error: String) -> Self {
        Error::from(error.as_str())
    }
}
/// Create a text error from a str slice.
impl From<&str> for Error {
    fn from(error: &str) -> Self {
        Error(format!("toolslib: {error}"))
    }
}
/// Create a text error from an `io::Error`.
impl From<text::Error> for Error {
    fn from(error: text::Error) -> Self {
        Error::from(error.to_string())
    }
}
