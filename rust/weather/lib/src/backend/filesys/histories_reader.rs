//! A threaded iterator over weather history archives.

use crate::{
    backend::{filesys::WeatherFile, WeatherDir},
    prelude::Location,
};
use std::{
    collections::VecDeque,
    sync::{
        mpsc::{channel, Receiver, Sender, RecvTimeoutError},
        Arc, Mutex,
    },
    thread, time,
};

/// The metadata used to create a [HistoryReader].
///
#[derive(Debug)]
pub struct HistoryReaderMD {
    /// The weather history location.
    pub location: Location,
    /// The weather history archive file.
    pub file: WeatherFile,
}

/// The thread-safe collection of [HistoryReader] metadata.
///
#[derive(Debug)]
pub struct HistoryReaderQueue(Mutex<VecDeque<HistoryReaderMD>>);
impl HistoryReaderQueue {
    /// Create the archive queue from the collection of locations.
    ///
    /// # Arguments
    ///
    /// * `weather_dir` is the weather data directory.
    /// * `locations` will be used to create the archive queue.
    ///
    pub fn new(weather_dir: &WeatherDir, locations: Vec<Location>) -> Self {
        let items = locations
            .into_iter()
            .map(|location| {
                let file = weather_dir.archive(&location.alias);
                HistoryReaderMD { location, file }
            })
            .collect();
        Self(Mutex::new(items))
    }
    /// Removes the next item from the queue or `None` if the queue is empty.
    ///
    pub fn take(&self) -> Option<HistoryReaderMD> {
        self.0.lock().unwrap_or_else(|err| err.into_inner()).pop_front()
    }
}

/// The API used by the [HistoriesReader] to read weather history archives.
///
pub trait HistoryReader<T> {
    /// Called by the [HistoriesReader] to start extracting archive information.
    fn read_archive(&self);
}

/// Generate the boilerplate code needed by most history readers.
macro_rules! generate_history_reader {
    ($name:ident, $type:ident) => {
        struct $name {
            /// The collection of location weather history metadata.
            queue: std::sync::Arc<crate::backend::filesys::histories_reader::HistoryReaderQueue>,
            /// The [Sender] side of the mpsc channel.
            sender: std::sync::mpsc::Sender<$type>,
        }
        impl $name {
            /// The factory method used to create [HistoryReader] instances.
            ///
            /// # Arguments
            ///
            /// * `queue` is the location weather history metadata.
            /// * `sender` is the [Sender] side of the mpsc channel.
            ///
            fn create(
                queue: std::sync::Arc<crate::backend::filesys::histories_reader::HistoryReaderQueue>,
                sender: std::sync::mpsc::Sender<$type>,
            ) -> Box<dyn crate::backend::filesys::histories_reader::HistoryReader<$type> + Send> {
                Box::new(Self { queue, sender })
            }
        }
    };
}
pub(in crate::backend) use generate_history_reader;

/// The iterator that will read multiple weather history archives in parallel.
///
pub struct HistoriesReader<T> {
    /// The current reader threads used by the reader.
    thread_handles: Vec<thread::JoinHandle<()>>,
    /// The receiving side of the MPSC channel.
    receiver: Receiver<T>,
    /// The time to wait for data to be sent.
    receiver_pause: time::Duration,
}
impl<T: Send + 'static> HistoriesReader<T> {
    /// Create the threaded history archive reader.
    ///
    /// # Arguments
    ///
    /// * `weather_dir` is the weather data directory.
    /// * `locations` identifies the weather history archives that will be read.
    /// * `max_threads` limits the number of history reader threads that will be used.
    /// * `reader_factor` creates a [HistoryReader] that will be used by each thread. Each reader
    /// is initialized with an instance of the [HistoryReaderQueue] and the mpsc [Sender].
    ///
    pub fn new<F>(weather_dir: &WeatherDir, locations: Vec<Location>, max_threads: usize, reader_factory: F) -> Self
    where
        F: Fn(Arc<HistoryReaderQueue>, Sender<T>) -> Box<dyn HistoryReader<T> + Send>,
    {
        let threads = std::cmp::min(locations.len(), max_threads);
        let queue = Arc::new(HistoryReaderQueue::new(weather_dir, locations));
        let (sender, receiver) = channel::<T>();
        // start up the threads that gather data
        let mut thread_handles = Vec::with_capacity(threads);
        for _ in 0..threads {
            let reader = reader_factory(queue.clone(), sender.clone());
            let handle = thread::spawn(move || {
                reader.read_archive();
            });
            thread_handles.push(handle);
        }
        // now that the threads are running close down this threads sender
        drop(sender);
        Self { thread_handles, receiver, receiver_pause: time::Duration::from_millis(1) }
    }
}
impl<T> Iterator for HistoriesReader<T> {
    type Item = T;
    /// Read data from the mspc [channel] until all data has been consumed and there are no more [HistoryReader]
    /// threads sending data.
    ///
    fn next(&mut self) -> Option<Self::Item> {
        // spin on the receiver until there's no one sending more data
        let mut next = None;
        loop {
            match self.receiver.recv_timeout(self.receiver_pause) {
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
impl<T> Drop for HistoriesReader<T> {
    /// When the [HistoriesReader] is done try to clean up the threads.
    ///
    fn drop(&mut self) {
        self.thread_handles.drain(..).for_each(|handle| {
            if !handle.is_finished() {
                log::warn!("HistoryReader {:?} did not finish.", handle.thread().id());
            } else if let Err(error) = handle.join() {
                log::error!("HistoryReader panicked: {error:?}");
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::testlib;

    #[test]
    fn queue() {
        let fixture = testlib::TestFixture::create();
        let weather_dir = WeatherDir::try_from(fixture.to_string()).unwrap();
        let aliases = vec!["one", "two", "three", "four"];
        let queue = HistoryReaderQueue::new(
            &weather_dir,
            aliases
                .iter()
                .map(|alias| Location {
                    city: "".to_string(),
                    state_id: "".to_string(),
                    state: "".to_string(),
                    name: "".to_string(),
                    alias: alias.to_string(),
                    latitude: "".to_string(),
                    longitude: "".to_string(),
                    tz: "".to_string(),
                })
                .collect(),
        );
        assert_eq!(queue.take().unwrap().location.alias, aliases[0]);
        assert_eq!(queue.take().unwrap().location.alias, aliases[1]);
        assert_eq!(queue.take().unwrap().location.alias, aliases[2]);
        assert_eq!(queue.take().unwrap().location.alias, aliases[3]);
        assert!(queue.take().is_none());
    }
}
