//! A collection of utilities to programmatically bootstrap `log4rs`.
//!
//! The log needs for the playground are mostly around being able to run
//! some command and exit. The [initialize] function will bootstrap `log4rs`
//! with a reasonable configuration and not require using an external
//! configuration file.
use log4rs::append::console::{ConsoleAppender, Target};
use log4rs::append::file::FileAppender;
use log4rs::config::runtime::{ConfigErrors, Logger};
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::filter::threshold::ThresholdFilter;
use log4rs::Handle;

use std::{fmt, io, path::PathBuf};

/// An error that can be returned from the `logs` module when initializing `log4rs`.
#[derive(Debug)]
pub struct LogError(String);

/// Satisfy the requirements for an error.
impl fmt::Display for LogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Consolidate the `log4rs` configuration error to a log error.
impl From<ConfigErrors> for LogError {
    fn from(error: ConfigErrors) -> Self {
        LogError(format!("{error}"))
    }
}

/// Consolidate the `log4rs` error when adding a logger to a log error.
impl From<log::SetLoggerError> for LogError {
    fn from(error: log::SetLoggerError) -> Self {
        LogError(format!("{error}"))
    }
}

/// Consolidate the standard IO Error to a log error.
impl From<io::Error> for LogError {
    fn from(error: io::Error) -> Self {
        LogError(format!("{error}"))
    }
}

/// The default appender pattern used by the console loggers.
#[cfg(not(debug_assertions))]
const DEFAULT_CONSOLE_PATTERN: &str = "{l:<5} {M} {m}{n}";
#[cfg(debug_assertions)]
const DEFAULT_CONSOLE_PATTERN: &str = "{l:<5} {f}:{L} {m}{n}";

/// The default appender pattern used by the file loggers.
#[cfg(not(debug_assertions))]
const DEFAULT_FILE_PATTERN: &str = "{d(%H:%M:%S%.3f)}|{l:<5}|{M} {m}{n}";
#[cfg(debug_assertions)]
const DEFAULT_FILE_PATTERN: &str = "{d(%H:%M:%S%.3f)}|{l:<5}|{f}:{L} {m}{n}";

/// The structure used to initialize `log4rs`.
pub struct LogProperties {
    /// The default log level that will be used.
    pub level: log::LevelFilter,
    /// The console logging pattern that will be used, if `None` the `DEFAULT_CONSOLE_PATTERN` will be used.
    pub console_pattern: Option<String>,
    /// The file logging pattern that will be used, if `None` the `DEFAULT_FILE_PATTERN` will be used.
    pub logfile_pattern: Option<String>,
    /// The path of the files log, if `None` logging to a file will not occur.
    pub logfile_path: Option<PathBuf>,
    /// Determines if logging output should be appended to the log file or not.
    pub logfile_append: bool,
    /// The loggers that will be associated with the file logger.
    pub file_loggers: Vec<String>,
}

/// Create log properties with default values.
///
/// The following defaults will be set.
///
/// * The logging level will be set to `Info`.
/// * The default console logging pattern will be used.
/// * The default file logging pattern will be used.
/// * The logfile path will be `None`.
/// * The append to log file attribute will be `false`.
/// * The collection of file loggers will be empty.
impl Default for LogProperties {
    fn default() -> Self {
        Self {
            level: log::LevelFilter::Info,
            console_pattern: None,
            logfile_pattern: None,
            logfile_path: None,
            logfile_append: false,
            file_loggers: vec![],
        }
    }
}
impl LogProperties {
    /// Used with default log properties to turn logging off.
    pub fn with_logging_off(mut self) -> Self {
        self.level = log::LevelFilter::Off;
        self
    }
    /// Used with default log properties to set `Error` level logging.
    pub fn with_error_logging(mut self) -> Self {
        self.level = log::LevelFilter::Error;
        self
    }
    /// Used with default log properties to set `Warn` level logging.
    pub fn with_warn_logging(mut self) -> Self {
        self.level = log::LevelFilter::Warn;
        self
    }
    /// Used with default log properties to set `Debug` level logging.
    pub fn with_debug_logging(mut self) -> Self {
        self.level = log::LevelFilter::Debug;
        self
    }
    /// Used with default log properties to set `Trace` level logging.
    pub fn with_trace_logging(mut self) -> Self {
        self.level = log::LevelFilter::Trace;
        self
    }
}

/// Initializes `log4rs` with a console logger (`stderr`) and an optional file logger.
///
/// On success it returns a handle to an object that allows the logging configuration to be
/// changed at runtime.
/// 
/// # Arguments
/// 
/// * `log_properties` the properties that help define the behaviour of `log4rs`.
pub fn initialize(log_properties: LogProperties) -> Result<Handle, LogError> {
    // the console appender goes to stderr
    let console_pattern = if let Some(pattern) = log_properties.console_pattern {
        pattern
    } else {
        String::from(DEFAULT_CONSOLE_PATTERN)
    };
    let console_encoder = PatternEncoder::new(&console_pattern);
    let console_appender = ConsoleAppender::builder().target(Target::Stderr).encoder(Box::new(console_encoder)).build();
    let console_log_level = if log_properties.logfile_path.is_none() {
        log_properties.level
    } else {
        // since there is a log file, filter console output separately
        match log_properties.level {
            log::LevelFilter::Debug | log::LevelFilter::Trace => log::LevelFilter::Info,
            _ => log_properties.level,
        }
    };
    // the configuration will always include the console appender
    let mut config_builder = Config::builder().appender(
        Appender::builder()
            .filter(Box::new(ThresholdFilter::new(console_log_level)))
            .build("console", Box::new(console_appender)),
    );

    // the root logger will always include the console appender
    let mut root_builder = Root::builder().appender("console");

    // the log pathname is the trigger to use a log file
    if let Some(log_pathname) = log_properties.logfile_path {
        // create the file appender
        let file_pattern = if let Some(pattern) = log_properties.logfile_pattern {
            pattern
        } else {
            String::from(DEFAULT_FILE_PATTERN)
        };
        let file_encoder = PatternEncoder::new(&file_pattern);
        let file_appender = FileAppender::builder()
            .append(log_properties.logfile_append)
            .encoder(Box::new(file_encoder))
            .build(log_pathname)?;

        // add the file appender to both configuration and root logger
        config_builder = config_builder.appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(log_properties.level)))
                .build("file", Box::new(file_appender)),
        );
        root_builder = root_builder.appender("file");

        // build the loggers that use the log file
        let loggers: Vec<Logger> = log_properties
            .file_loggers
            .iter()
            .map(|logger| Logger::builder().appender("file").additive(false).build(logger, log_properties.level))
            .collect();

        // add the file loggers to the configuration
        config_builder = config_builder.loggers(loggers);
    }
    let config = config_builder.build(root_builder.build(log_properties.level))?;
    // eprintln!("{:?}", config);
    let handle = log4rs::init_config(config)?;
    Ok(handle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::{debug, error, info, trace, warn};
    #[test]
    fn initialize() {
        // super::initialize(3, Some(PathBuf::from("test.log")), false).unwrap();
        super::initialize(LogProperties {
            level: log::LevelFilter::Info,
            console_pattern: None,
            logfile_pattern: None,
            logfile_path: Some(PathBuf::from("test.log")),
            logfile_append: false,
            file_loggers: vec![String::from("toolslib::logs::tests")],
        })
        .unwrap();
        error!("error message");
        warn!("warn message");
        info!("info message");
        debug!("debug message");
        trace!("trace message");
    }
}
