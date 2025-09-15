//! # A timer allowing elapsed time to be tracked.
//!
//! Yeah, yeah, yeah. There are lots of these around but this is the
//! type of API I'm use to so here it is.
use std::fmt;
use std::fmt::Formatter;
use std::time::{Duration, Instant};

/// The stopwatch data.
#[derive(Debug)]
pub struct StopWatch {
    /// When the stop watch was started or `None`.
    start: Option<Instant>,
    /// How long the stopwatch was run or `None`
    duration: Option<Duration>,
}

/// How the stopwatch should be displayed.
impl fmt::Display for StopWatch {
    /// The default is to display the stop watch in milliseconds.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use thousands::Separable;
        write!(f, "{}ms", self.millis().separate_with_commas())
    }
}

impl StopWatch {
    /// Returns a new instance of the stopwatch.
    pub fn new() -> StopWatch {
        StopWatch {
            start: None,
            duration: None,
        }
    }
    /// Returns a new instance of the stopwatch that has been started.
    pub fn start_new() -> StopWatch {
        StopWatch {
            start: Some(Instant::now()),
            duration: None,
        }
    }
    /// Starts or re-starts the stopwatch.
    pub fn start(&mut self) {
        self.start = Some(Instant::now());
        self.duration = None;
    }
    /// Stops the stopwatch.
    ///
    /// If the stop watch has not been started the duration will be set to 0 seconds.
    pub fn stop(&mut self) {
        match self.start {
            Some(start) => {
                self.duration = Some(Instant::now() - start);
                self.start = None
            }
            None => self.duration = Some(Duration::from_secs(0)),
        }
    }
    /// Reset the stopwatch to it's initial values.
    pub fn reset(&mut self) -> &mut Self {
        self.start = None;
        self.duration = None;
        self
    }
    pub fn time_str(&self) -> String {
        let overall_millis = self.millis();
        let millis = overall_millis % 1000;
        let seconds = (overall_millis / 1000) % 60;
        let minutes = (overall_millis / 1000 / 60) % 60;
        let hours = overall_millis / 1000 / 60 / 60;
        format!("{}:{:0>2}:{:0>2}.{:0>3}", hours, minutes, seconds, millis)
    }
    /// Returns the duration recorded in the stopwatch.
    pub fn elapsed(&self) -> Duration {
        if let Some(start) = self.start {
            Instant::now() - start
        } else if let Some(duration) = self.duration {
            duration
        } else {
            Duration::from_secs(0)
        }
    }
    /// Returns true if the stopwatch has been started.
    pub fn is_running(&self) -> bool {
        return self.start.is_some();
    }
    /// Returns how long the stop watch has been running.
    pub fn millis(&self) -> i64 {
        return self.elapsed().as_millis() as i64;
    }
}
