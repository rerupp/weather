//! The history archive file writer.
//!

use super::{archive::date_to_filename, ArchiveData};
use crate::backend::filesys::WeatherFile;
use chrono::{Datelike, Timelike, Utc};
use std::fs::{self, File};
use std::io::Write;
use zip::{write::SimpleFileOptions, CompressionMethod, DateTime, ZipWriter};

/// The extension that identifies a writable archive.
pub const UPDATE_EXT: &'static str = "upd";

/// The extension that identifies an archive backup.
pub const BACKUP_EXT: &'static str = "bu";

/// The [ArchiveWriter] error builder.
///
macro_rules! error {
    ($id:expr, $reason:expr) => {
        crate::Error::from(format!("'{}' ArchiveWriter {}", $id, $reason))
    };
}

/// The manager that adds weather history to an archive.
#[derive(Debug)]
pub struct ArchiveWriter<'w> {
    /// The archive alias.
    lid: String,
    /// The archive that will be updated.
    archive: &'w WeatherFile,
}
impl<'w> ArchiveWriter<'w> {
    /// Create a new instance of the archive writer.
    ///
    /// # Arguments
    ///
    /// `archive` is the weather history archive.
    ///
    pub fn new(lid: &str, archive: &'w WeatherFile) -> Self {
        // Self { archive, writable }
        Self { lid: lid.to_string(), archive }
    }

    /// Adds history to the archive.
    ///
    /// # Arguments
    ///
    /// `histories` is what will be added to the archive.
    pub fn add_data(&mut self, histories: Vec<ArchiveData>) -> crate::Result<()> {
        let mut writer = self.open()?;
        for file_data in histories {
            self.write_file(&mut writer, file_data)?;
        }
        self.close(writer)
    }

    /// Writes history into the archive.
    ///
    /// # Arguments
    ///
    /// * `writer` will be used to add the history.
    /// * `date` is the data associated with the history.
    /// * `data` is the history serialized into a sequence of bytes.
    fn write_file(&self, writer: &mut ZipWriter<File>, file_data: ArchiveData) -> crate::Result<()> {
        let now = Utc::now().naive_utc();
        let mtime = DateTime::from_date_and_time(
            now.year() as u16,
            now.month() as u8,
            now.day() as u8,
            now.hour() as u8,
            now.minute() as u8,
            now.second() as u8,
        )
        // this should never fail unless time is snafu
        .unwrap();
        let filename = date_to_filename(&self.lid, &file_data.date);
        let options =
            SimpleFileOptions::default().compression_method(CompressionMethod::Deflated).last_modified_time(mtime);
        if let Err(start_error) = writer.start_file(&filename, options) {
            Err(error!(self.lid, format!("failed to start history on {}: {}.", file_data.date, start_error)))
        } else if let Err(write_error) = writer.write_all(&file_data.data) {
            Err(error!(self.lid, format!("failed to write history on {}: {}", file_data.date, write_error)))
        } else {
            Ok(())
        }
    }

    /// Creates the [ZipWriter] that will update the archive.
    ///
    /// In order to add data the archive is first copied to the writable path. When done adding history the
    /// archive will be restored when the [ZipWriter] is closed.
    ///
    fn open(&self) -> crate::Result<ZipWriter<File>> {
        let update_file = self.archive.with_extension(UPDATE_EXT);
        if let Err(error) = self.archive.copy(&update_file) {
            Err(error)?;
        }
        match File::options().read(true).write(true).open(update_file.path()) {
            Ok(file) => match ZipWriter::new_append(file) {
                Ok(zip_writer) => Ok(zip_writer),
                Err(zip_error) => Err(error!(self.lid, format!("failed to open update file: {}", zip_error))),
            },
            Err(file_error) => Err(error!(self.lid, format!("failed to create update file: {}.", file_error))),
        }
    }

    /// Close the [ZipWriter] and restore the archive.
    ///
    /// When the archive is opened a copy is made and a [ZipWriter] returned that will be used. After it
    /// is closed, the updated archive replaces the original.
    ///
    /// # Arguments
    ///
    /// * `writer` is what was used to update the archive histories.
    ///
    fn close(&self, writer: ZipWriter<File>) -> crate::Result<()> {
        // close the writer now to flush contents
        // drop(writer);
        if let Err(finish_error) = writer.finish() {
            Err(error!(self.lid, format!("failed to finish archive update: {}", finish_error)))?;
        }
        // try to safely replace the updated archive
        let update_file = self.archive.with_extension(UPDATE_EXT);
        let backup_file = self.archive.with_extension(BACKUP_EXT);
        // make a copy of the archive
        if let Err(error) = self.archive.copy(&backup_file) {
            Err(error!(self.lid, format!("failed to create backup file: {}.", error)))?;
        }
        // try to safe update the archive
        match update_file.rename(&self.archive) {
            Ok(_) => {
                // remove the backed up file
                if let Err(remove_error) = backup_file.remove() {
                    log::warn!("Error removing archive backup {}: {}", backup_file, remove_error);
                }
                Ok(())
            }
            Err(update_error) => {
                // try to restore the original archive
                if let Err(recover_error) = backup_file.rename(&self.archive) {
                    log::error!(
                        "Error updating {} and it as not be recovered!\nUpdate: {}\nrecover error: {}",
                        self.archive,
                        update_error,
                        recover_error
                    );
                    Err(error!(self.lid, "failed to close archive and the backup file was not recovered!"))
                } else {
                    Err(error!(self.lid, format!("failed to update archive: {}.", update_error)))
                }
            }
        }
    }

    // /// Copy the contents of one archive to another (see std::fs::copy).
    // ///
    // /// Arguments:
    // ///
    // /// * `from` is the source file.
    // /// * `to` is the destination file.
    // ///
    // fn copy_archive(&self, from: &Path, to: &Path) -> crate::Result<()> {
    //     // copy will overwrite the target file if it exists
    //     if let Err(error) = fs::copy(from, to) {
    //         Err(error!(self.lid, format!("error copying {} to {}: {}", from.display(), to.display(), &error)))
    //     } else {
    //         Ok(())
    //     }
    // }
}
impl<'w> Drop for ArchiveWriter<'w> {
    /// If something bad happens adding history, this attempts to clean up files that might be
    /// left around.
    fn drop(&mut self) {
        // do your best to clean up
        let master_file = self.archive.path();
        let update_file = master_file.with_extension(UPDATE_EXT);
        if update_file.exists() {
            if let Err(error) = fs::remove_file(&update_file) {
                log::warn!("ArchiveWriter::drop(): error deleting {}: {}.", update_file.display(), error);
            }
        }
        let backup_file = master_file.with_extension(BACKUP_EXT);
        if backup_file.exists() {
            if let Err(error) = fs::remove_file(&backup_file) {
                log::warn!("ArchiveWriter::drop(): error deleting {}: {}.", backup_file.display(), error);
            }
        }
    }
}
