//! A threaded iterator over weather history archives.

use crate::{
    backend::{filesys::WeatherFile, WeatherDir},
    prelude::Location,
};
use std::{
    sync::{
        mpsc::{channel, Receiver, RecvTimeoutError, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

/// The metadata used by an [ArchivesReader] to collect archive information.
///
#[derive(Debug)]
pub struct ArchivesReaderCtx {
    /// The weather history location.
    pub location: Location,
    /// The weather history archive file.
    pub file: WeatherFile,
}

/// The API used by the [ArchivesIterator] to read weather history archives.
///
pub trait ArchivesReader<T> {
    /// Called by the [ArchivesIterator] to collect archive information.
    ///
    /// # Arguments
    ///
    /// * `ctx` identifies which location and weather data file to read.
    ///
    fn read_archive(&self, ctx: ArchivesReaderCtx);
}

/// A multithreaded iterator that uses [ArchivesReader] instances to collect weather
/// history archive information.
///
pub struct ArchivesIterator<T> {
    /// What the iterator uses to read the weather history archive data.
    receiver: Receiver<T>,
    /// The worker thread handles.
    thread_handles: Vec<JoinHandle<()>>,
    /// How long the receiver waits for data before trying again.
    timeout: Duration,
}
impl<T: 'static> ArchivesIterator<T> {
    /// Create a new instance of the iterator.
    ///
    /// # Arguments
    ///
    /// * `weather_dir` is the weather history data directory.
    /// * `locations` is the collection of weather history locations that will be iterated across.
    /// * `threads` establishes how many readers will be used.
    /// * `reader_factory` is used to create a worker readers.
    ///
    pub fn new<F>(weather_dir: &WeatherDir, locations: Vec<Location>, threads: usize, reader_factory: F) -> Self
    where
        F: Fn(Sender<T>) -> Box<dyn ArchivesReader<T> + Send>,
    {
        // set up the maximum threads to use
        let workers = std::cmp::min(threads, locations.len());

        // create the collection of reader context
        let mut readers_ctx = locations
            .into_iter()
            .map(|location| {
                let file = weather_dir.archive(&location.alias);
                ArchivesReaderCtx { location, file }
            })
            .collect::<Vec<_>>();

        // reverse the collection so it will be pulled in location sort order
        readers_ctx.reverse();

        // start up the workers
        let (sender, receiver) = channel::<T>();
        let queue = Arc::new(Mutex::new(readers_ctx));
        let mut thread_handles = Vec::with_capacity(workers);
        for _ in 0..workers {

            // create what the worker thread needs to run
            let inner_queue = queue.clone();
            let inner_reader = reader_factory(sender.clone());

            // the worker thread that manages calling the archive reader
            let handle = thread::spawn(move || {
                loop {
                    // get in and out of the lock quickly
                    let ctx_opt = inner_queue.lock().unwrap_or_else(|err| err.into_inner()).pop();
                    match ctx_opt {
                        Some(ctx) => inner_reader.read_archive(ctx),
                        None => break,
                    }
                }
            });

            // save the thread handles so they can be joined when the iterator is dropped
            thread_handles.push(handle);
        }
        Self { receiver, thread_handles, timeout: Duration::from_millis(1) }
    }
}
impl<T> Iterator for ArchivesIterator<T> {
    type Item = T;
    /// Read data from the mspc [channel] until all the threads are done sending data.
    fn next(&mut self) -> Option<Self::Item> {
        // spin on the receiver until the threads shutdown
        let mut next = None;
        loop {
            match self.receiver.recv_timeout(self.timeout) {
                Ok(t) => {
                    next.replace(t);
                    break;
                }
                Err(err) => match err {
                    RecvTimeoutError::Timeout => (),
                    // there are no more senders so the iterator is done
                    RecvTimeoutError::Disconnected => break,
                },
            }
        }
        next
    }
}
impl<T> Drop for ArchivesIterator<T> {
    /// Try to clean up the threads once the iterator is done.
    ///
    fn drop(&mut self) {
        self.thread_handles.drain(..).for_each(|handle| {
            if !handle.is_finished() {
                log::warn!("HistoryArchivesIterator {:?} did not finish.", handle.thread().id());
            } else if let Err(error) = handle.join() {
                log::error!("HistoryArchivesIterator panicked: {error:?}");
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::testlib;

    struct TestcaseReader {
        sender: Sender<u32>,
    }
    impl TestcaseReader {
        pub fn factory(sender: Sender<u32>) -> Box<dyn ArchivesReader<u32> + Send> {
            Box::new(Self { sender })
        }
    }
    impl ArchivesReader<u32> for TestcaseReader {
        fn read_archive(&self, ctx: ArchivesReaderCtx) {
            let value = ctx.location.latitude.parse::<u32>().unwrap();
            match self.sender.send(value) {
                Ok(_) => (),
                Err(error) => eprintln!("{error:?}"),
            }
        }
    }
    #[test]
    fn queue() {
        let fixture = testlib::TestFixture::create();
        let weather_dir = WeatherDir::try_from(fixture.to_string()).unwrap();
        let location = |alias: &str, lat: &str| Location {
            alias: alias.to_string(),
            latitude: lat.to_string(),
            ..Default::default()
        };
        let locations = vec![
            location("one", "1"),
            location("two", "2"),
            location("three", "3"),
            location("four", "4"),
            location("five", "5"),
            location("six", "6"),
            location("seven", "7"),
            location("eight", "8"),
            location("nine", "9"),
            location("ten", "10"),
        ];
        let mut lats =
            ArchivesIterator::new(&weather_dir, locations, 3, TestcaseReader::factory).collect::<Vec<u32>>();
        lats.sort();
        for idx in 0..lats.len() {
            assert_eq!(lats[idx], idx as u32 + 1);
        }
    }
}
