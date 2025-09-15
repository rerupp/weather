//! The filesys module admin API
//!
pub(in crate::backend) use v2::filesys_details;
mod v2 {
    //! The current implementation of administration for the file system.
    use crate::{
        admin_prelude::{FilesysDetails, LocationDetails},
        backend::filesys::{HistoryArchive, Locations, WeatherDir}
    };

    pub fn filesys_details(weather_dir: &WeatherDir) -> crate::Result<FilesysDetails> {
        let mut location_details = vec![];
        let mut archives_size: u64 = 0;
        // for location in locations {
        for location in Locations::open(weather_dir)?.get()? {
            let file = weather_dir.archive(&location.alias);
            archives_size += file.size();
            let mut histories: usize = 0;
            let compressed_size: usize = HistoryArchive::open(&location.alias, file)?
                .metadata()?
                .map(|metadata| {
                    histories += 1;
                    metadata.compressed_size as usize
                })
                .sum();
            location_details.push(LocationDetails { alias: location.alias.clone(), size: compressed_size, histories })
        }
        Ok(FilesysDetails { size: archives_size as usize, location_details })
    }
}
