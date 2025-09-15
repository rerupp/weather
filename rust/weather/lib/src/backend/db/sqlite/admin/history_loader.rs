//! The normalized database archive loader for [History].
//!
use super::{history::insert_history, locations};
use crate::{
    backend::{
        db::sqlite::estimate_size,
        filesys::{ArchiveMetadata, HistoryArchive, WeatherDir, WeatherFile},
    },
    entities::History,
};
use rusqlite::Connection;
use std::{
    marker::PhantomData,
    sync::{
        mpsc::{channel, Receiver, Sender, TryRecvError},
        Arc, Mutex,
    },
    thread, time,
};
use toolslib::stopwatch::StopWatch;

/// Create a load history specific error message.
macro_rules! error {
    ($($arg:tt)*) => {
        crate::Error::from(format!("loader {}", format!($($arg)*)))
    }
}

/// Create an error from the load history specific error message.
macro_rules! err {
    ($($arg:tt)*) => {
        Err(error!($($arg)*))
    };
}

/// The data passed through the [ArchiveLoader].
/// 
#[derive(Debug)]
struct LoadMsg {
    /// The location table identifier.
    lid: i64,
    /// The archive metadata.
    md: ArchiveMetadata,
    /// The daily history.
    history: History,
}

/// Take the [History] archives and push them into the database.
///
/// # Argument
///
/// * `conn` is the database connection that will be used.
/// * `weather_dir` is the weather data directory.
/// * `threads` is the number of workers to use getting data from archives.
///
pub fn load(conn: Connection, weather_dir: &WeatherDir, threads: usize) -> crate::Result<()> {
    // let conn = db_conn!(weather_dir)?;
    let size_estimate = estimate_size(&conn, "history")?;
    let archives = ArchiveQueue::new(&conn, weather_dir)?;
    let mut loader: ArchiveLoader<LoadMsg> = ArchiveLoader::new(threads);
    loader.execute(
        archives,
        || Box::new(HistoryProducer),
        || Box::new(HistoryConsumer { conn, base_size: size_estimate }),
    )
}

/// The [History] data producer.
struct HistoryProducer;
impl HistoryProducer {
    /// Send the history data to the consumer side of the loader.
    ///
    /// # Arguments
    ///
    /// * `lid` is the locations primary id in the database.
    /// * `history` is the data that will be sent off to the consumer.
    /// * `sender` is used to pass data to the collector.
    /// 
    fn send_history(
        &self,
        lid: i64,
        md: ArchiveMetadata,
        history: History,
        sender: &Sender<LoadMsg>,
    ) -> crate::Result<()> {
        let msg = LoadMsg { lid, md, history };
        if let Err(error) = sender.send(msg) {
            err!("failed sending history to consumer: {:?}", error)?;
        }
        Ok(())
    }
}
impl ArchiveProducer<LoadMsg> for HistoryProducer {
    /// This is called by the archive producer to get data from the archive.
    ///
    /// # Arguments
    ///
    /// * `lid` is the locations primary id in the database.
    /// * `alias` is the locations alias name.
    /// * `file` is the weather data archive.
    /// * `sender` is used to pass data to the collector.
    ///
    fn gather(&self, lid: i64, alias: &str, file: WeatherFile, sender: &Sender<LoadMsg>) -> crate::Result<usize> {
        let mut history_count = 0;
        let metadata_and_history = HistoryArchive::open(alias, file)?.metadata_and_history()?;
        for (metadata, history) in metadata_and_history {
            self.send_history(lid, metadata, history, sender)?;
            history_count += 1;
        }
        Ok(history_count)
    }
}

/// The database history loader.
/// 
struct HistoryConsumer {
    /// The database connection that will be used.
    conn: Connection,
    /// The base size of a row minus the text field lengths.
    base_size: usize,
}
impl ArchiveConsumer<LoadMsg> for HistoryConsumer {
    /// Called by the [ArchiveLoader] to collect the weather history being mined.
    ///
    /// # Arguments
    ///
    /// * `receiver` is used to collect the weather data.
    /// 
    fn collect(&mut self, receiver: Receiver<LoadMsg>) -> crate::Result<usize> {
        // create the transaction
        let mut tx = match self.conn.transaction() {
            Ok(tx) => tx,
            Err(error) => err!("consumer failed to create transaction: {:?}", error)?,
        };
        let mut count: usize = 0;

        // spin on the receiver until there's no one sending more data
        let pause = time::Duration::from_millis(1);
        loop {
            match receiver.try_recv() {
                Ok(msg) => {
                    let mut size = self.base_size + msg.history.description.as_ref().map_or(0, |s| s.len());
                    size += msg.history.precipitation_type.as_ref().map_or(Default::default(), |t| t.len());
                    insert_history(&mut tx, msg.lid, size, msg.md.compressed_size as usize, &msg.history)?;
                    count += 1;
                }
                Err(err) => match err {
                    TryRecvError::Empty => thread::sleep(pause),
                    // there are no more senders to you are done
                    TryRecvError::Disconnected => break,
                },
            }
        }

        if let Err(error) = tx.commit() {
            err!("failed to commit transaction: {:?}", error)?;
        }
        Ok(count)
    }
}

/// The archive metadata used by the [crate::backend::db::archive::loader::ArchiveQueue].
#[derive(Debug)]
pub struct ArchiveQueueMd {
    /// The database primary id of the weather location.
    pub lid: i64,
    /// The weather location alias name.
    pub alias: String,
    /// The weather data archive.
    pub file: WeatherFile,
}

/// A thread-safe collection of weather archive metadata used by the [ArchiveLoader].
/// 
#[derive(Debug)]
pub struct ArchiveQueue(Mutex<Vec<ArchiveQueueMd>>);
impl ArchiveQueue {
    pub fn new(conn: &Connection, weather_dir: &WeatherDir) -> crate::Result<Self> {
        let id_alias_files: Vec<ArchiveQueueMd> = locations::id_aliases(conn)?
            .into_iter()
            .map(|(lid, alias)| {
                let file = weather_dir.archive(&alias);
                ArchiveQueueMd { lid, alias, file }
            })
            .collect();
        Ok(Self(Mutex::new(id_alias_files)))
    }
    pub fn next(&self) -> Option<ArchiveQueueMd> {
        match self.0.lock() {
            Ok(mut guard) => guard.pop(),
            Err(err) => err.into_inner().pop(),
        }
    }
}

/// The trait used by the [ArchiveLoader] to gather data from a weather archive.
/// 
pub trait ArchiveProducer<T> {
    /// The *producer* side of the archive data.
    ///
    /// # Arguments
    ///
    /// * `lid` is the database location id.
    /// * `alias` is the location alias name.
    /// * `archive` is the locations weather data archive file.
    /// * `sender` is used to hand off the gathered archive data.
    ///
    fn gather(&self, lid: i64, alias: &str, archive: WeatherFile, sender: &Sender<T>) -> crate::Result<usize>;

    /// Trait boilerplate that gets archive metadata from the queue and calls the data extractor.
    ///
    /// # Arguments
    ///
    /// * `sender` gathers archive history.
    /// * `archives` is a thread safe queue that collects the gathered history.
    ///
    fn send(&self, sender: Sender<T>, archives: Arc<ArchiveQueue>) {
        while let Some(md) = archives.next() {
            let mut load_time = StopWatch::start_new();
            let filename = md.file.filename.clone();
            match self.gather(md.lid, &md.alias, md.file, &sender) {
                Ok(count) => {
                    load_time.stop();
                    self.log_elapsed(&md.alias, count, &load_time);
                }
                Err(err) => {
                    log::error!("{:?} error loading archive {} ({}).", thread::current().id(), filename, &err);
                    break;
                }
            }
        }
    }

    /// Trait boilerplate that logs elapsed time for the producer.
    ///
    /// # Arguments
    ///
    /// * `description` tersely describes the elapsed time.
    /// * `count` is the number of items mined from the archive.
    /// * `load_time` is how long the gather took.
    ///
    fn log_elapsed(&self, description: &str, count: usize, load_time: &StopWatch) {
        log_elapsed(description, count, load_time);
    }
}

/// The trait used by the [ArchiveLoader] to collect the data gathered from weather archives.
///
pub trait ArchiveConsumer<T> {
    /// The *consumer* side of the archive data.
    ///
    /// # Arguments
    ///
    /// * `receiver` is used to collect the gathered archive data.
    ///
    fn collect(&mut self, receiver: Receiver<T>) -> crate::Result<usize>;

    /// The boilerplate side for the *consumer* of archive data.
    ///
    /// # Arguments
    ///
    /// * `receiver` is used to collect the gathered archive data.
    ///
    fn receive(&mut self, receiver: Receiver<T>) {
        let mut load_time = StopWatch::start_new();
        match self.collect(receiver) {
            Ok(count) => {
                load_time.stop();
                self.log_elapsed("Overall", count, &load_time);
            }
            Err(err) => {
                log::error!("{}", format!("ArchiveConsumer collect error ({})", &err));
            }
        }
    }
    /// Trait boilerplate that logs elapsed time for the consumer.
    ///
    /// # Arguments
    ///
    /// * `description` tersely describes the elapsed time.
    /// * `count` is the number of items mined from the archive.
    /// * `load_time` is how long the collection took.
    ///
    fn log_elapsed(&self, description: &str, count: usize, load_time: &StopWatch) {
        log_elapsed(description, count, load_time);
    }
}

/// The default logger used by the ArchiveConsumer and ArchiveSender traits.
///
/// # Arguments
///
/// * `description` tersely describes the elapsed time.
/// * `count` is the number of items mined from the archive.
/// * `load_time` is how long the collection took.
///
#[inline]
fn log_elapsed(description: &str, count: usize, load_time: &StopWatch) {
    log::debug!(
        "{:?} {}: {} loaded in {} ({:0.3}history/ms).",
        thread::current().id(),
        description,
        toolslib::fmt::commafy(count),
        load_time,
        count as f64 / load_time.millis() as f64
    );
}

/// A threaded framework that gathers data from archives.
#[derive(Debug)]
pub struct ArchiveLoader<T> {
    /// The number of threads to use.
    threads: usize,
    /// The **`I need to be associated with a type`** compiler hack.
    phantom: PhantomData<T>,
}
impl<T: 'static + Send> ArchiveLoader<T> {
    /// Create a new instance of the loader.
    ///
    /// # Arguments
    ///
    /// * `threads` is the number of threads to use gathering data.
    /// 
    pub fn new(threads: usize) -> ArchiveLoader<T> {
        Self { threads, phantom: PhantomData }
    }
    
    /// Gather data from a collection of archives.
    ///
    /// # Arguments
    ///
    /// * `archives` is the collection of archives data will be gathered from.
    /// * `producer` is used to create the threads that gather archive data.
    /// * `consumer` is used to create the collector of archive data.
    /// 
    pub fn execute<P, C>(&mut self, archives: ArchiveQueue, producer: P, consumer: C) -> crate::Result<()>
    where
        P: Fn() -> Box<dyn ArchiveProducer<T> + Send>,
        C: FnOnce() -> Box<dyn ArchiveConsumer<T> + Send>,
    {
        // start up the threads that gather data
        let archives = Arc::new(archives);
        let (sender, receiver) = channel::<T>();
        let mut handles = Vec::with_capacity(self.threads);
        for _ in 0..self.threads {
            let producer = producer();
            let sender = sender.clone();
            let archive_queue = archives.clone();
            let handle = thread::spawn(move || {
                producer.send(sender, archive_queue);
            });
            handles.push(handle);
        }
        // now that the threads are running close down the sender
        drop(sender);
        // run the consumer
        consumer().receive(receiver);
        // now cleanup the threads
        for handle in handles {
            let thread_id = handle.thread().id();
            match handle.join() {
                Ok(_) => (),
                Err(_) => {
                    log::error!("Error joining with thread ({:?})", thread_id);
                }
            }
        }
        Ok(())
    }
}
