use super::validate;
use crate::backend::filesys::{WeatherDir, WeatherFile};
use crate::entities::Location;
use serde::{Deserialize, Serialize};
use std::io::{BufWriter, Write};

/// The name of the locations document in the weather data directory.
const LOCATIONS_FILENAME: &'static str = "locations.json";

/// The name of the updated locations document in the weather data directory.
const UPDATE_EXTENSION: &'static str = "upd";

/// The name of the backup locations document in the weather data directory.
const BACKUP_EXTENSION: &'static str = "bck";

/// Creates a locations document error.
macro_rules! error {
    ($message:expr) => {
        crate::Error::from(format!("LocationsDocument {}", $message))
    };
}

/// Create an Err for the error.
macro_rules! err {
    ($($arg:tt)*) => {
        Err(error!(format!($($arg)*)))
    }
}

/// The locations `JSON` document manager.
#[derive(Debug)]
pub struct LocationsFile {
    file: WeatherFile,
}
impl LocationsFile {
    /// Tests if the location file exists in the weather directory.
    ///
    /// Arguments
    ///
    /// * `weather_dir` is the weather directory.
    ///
    pub fn exists(weather_dir: &WeatherDir) -> bool {
        weather_dir.file(LOCATIONS_FILENAME).exists()
    }

    /// Opens the location file in the weather directory returning an error if the file does not
    /// exist.
    ///
    /// Arguments
    ///
    /// * `weather_dir` is the weather directory.
    ///
    pub fn open(weather_dir: &WeatherDir) -> crate::Result<Self> {
        let file = WeatherFile::from(weather_dir.file(LOCATIONS_FILENAME));
        match file.exists() {
            true => Ok(Self { file }),
            false => err!("{} does not exist.", file),
        }
    }

    /// Create the location file in the weather directory returning an error if the file already
    /// exist.
    ///
    /// Arguments
    ///
    /// * `weather_dir` is the weather directory.
    ///
    pub fn create(weather_dir: &WeatherDir) -> crate::Result<Self> {
        let file = WeatherFile::from(weather_dir.file(LOCATIONS_FILENAME));
        match file.exists() {
            true => err!("{} already exist.", file),
            false => {
                let self_ = Self { file };
                self_.save(vec![])?;
                Ok(self_)
            }
        }
    }

    /// Read the contents of the location file.
    ///
    pub fn load(&self) -> crate::Result<impl Iterator<Item = LocationDocument>> {
        let reader = self.file.reader()?;
        let result: Result<LocationDocuments, serde_json::Error> = serde_json::from_reader(reader);
        match result {
            Err(error) => err!("failed to load locations from {}: {:?}", self.file, error),
            Ok(mut documents) => {
                documents = documents.validate_and_dedup();
                Ok(documents.into_iter())
            }
        }
    }

    /// Replace the location file with the location documents.
    ///
    /// Arguments
    ///
    /// * `documents` replaces the locations file contents.
    ///
    pub fn save(&self, documents: Vec<LocationDocument>) -> crate::Result<()> {
        // make sure the update file doesn't exist
        let update_file = self.file.with_extension(UPDATE_EXTENSION);
        if update_file.exists() {
            update_file.remove()?;
        }

        // write the new locations document
        update_file.touch()?;
        let mut writer = BufWriter::new(update_file.writer()?);
        let location_documents = LocationDocuments { locations: documents };
        if let Err(write_error) = serde_json::to_writer_pretty(&mut writer, &location_documents) {
            err!("failed write to locations file {}: {:?}", self.file, write_error)
        } else if let Err(flush_error) = writer.flush() {
            err!("failed flush on locations file {}: {:?}", self.file, flush_error)
        } else {
            // replace the locations document
            drop(writer);
            let backup_file = self.file.with_extension(BACKUP_EXTENSION);
            if self.file.exists() {
                self.file.copy(&backup_file)?;
            }
            update_file.rename(&self.file)?;
            if backup_file.exists() {
                if let Err(error) = backup_file.remove() {
                    // don't throw an error if the backup file cannot be removed
                    log::warn!("error removing backup file: {:?}", error);
                }
            }
            Ok(())
        }
    }
}

/// The bean that describes the locations `JSON` document.
#[derive(Debug, Deserialize, Serialize)]
struct LocationDocuments {
    /// The collection of location metadata.
    locations: Vec<LocationDocument>,
}
impl LocationDocuments {
    /// Scan the collection of documents to make sure they are valid. Locations with duplicate
    /// alias names will be removed except for the first locations. The collection will be in
    /// location name order when this completes.
    ///
    fn validate_and_dedup(self) -> Self {
        let mut locations: Vec<LocationDocument> = Vec::with_capacity(self.locations.len());
        for (index, mut location) in self.locations.into_iter().enumerate() {
            if location.name.is_empty() {
                location.name = format!("{}, {}", location.city, location.state_id);
            }
            if location.ok(index) {
                locations.push(location);
            }
        }
        // order the documents by alias to remove duplicates
        locations.sort_by(|lhs, rhs| lhs.alias.cmp(&rhs.alias));
        locations.dedup_by(|lhs, rhs| {
            let duplicate = lhs.alias == rhs.alias;
            if duplicate {
                log::warn!("The alias name '{}' for {} is already being used.", lhs.alias, lhs.name)
            }
            duplicate
        });
        // finally order the documents by name
        locations.sort_unstable_by(|lhs, rhs| lhs.name.cmp(&rhs.name));
        Self { locations }
    }
}
impl IntoIterator for LocationDocuments {
    type Item = LocationDocument;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.locations.into_iter()
    }
}

/// Write a warning message to the log file.
macro_rules! warn {
    ($index: expr, $($arg:tt)*) => {
        log::warn!("problem with document at index {}: {}", $index, error!(format!($($arg)*)))
    }
}

/// The bean that describes the metadata for a location.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct LocationDocument {
    /// The location city name.
    pub city: String,
    /// The location abbreviated state name.
    pub state_id: String,
    /// The location full state name
    pub state: String,
    /// The name of a location does not persist.
    #[serde(skip)]
    pub name: String,
    /// A unique nickname of a location.
    pub alias: String,
    /// The location latitude.
    pub latitude: String,
    /// The location longitude.
    pub longitude: String,
    /// the location timezone.
    pub tz: String,
    /// The validation flag does not persist.
    #[serde(skip)]
    valid: bool,
}
impl LocationDocument {
    /// Verify the location document is valid.
    ///
    /// Arguments
    ///
    /// * `index` is the position of the location within the JSON document.
    ///
    pub fn ok(&mut self, index: usize) -> bool {
        self.valid = true;

        match validate::city(&self.city) {
            Ok(name) => self.city = name,
            Err(error) => {
                warn!(index, "{}", error);
                self.valid = false;
            }
        }

        match validate::state_id(&self.state_id) {
            Ok(name) => self.state_id = name,
            Err(error) => {
                warn!(index, "{}", error);
                self.valid = false;
            }
        }

        match validate::state(&self.state) {
            Ok(name) => self.state = name,
            Err(error) => {
                warn!(index, "{}", error);
                self.valid = false;
            }
        }

        match validate::alias(&self.alias) {
            Ok(alias) => self.alias = alias,
            Err(error) => {
                warn!(index, "{}", error);
                self.valid = false;
            }
        }

        match validate::latitude(&self.latitude) {
            Ok(latitude) => self.latitude = latitude,
            Err(error) => {
                warn!(index, "{}", error);
                self.valid = false;
            }
        }

        match validate::longitude(&self.longitude) {
            Ok(longitude) => self.longitude = longitude,
            Err(error) => {
                warn!(index, "{}", error);
                self.valid = false;
            }
        }

        match validate::tz(&self.tz) {
            Ok(tz) => self.tz = tz,
            Err(error) => {
                warn!(index, "{}", error);
                self.valid = false;
            }
        }
        self.valid
    }
}
impl From<LocationDocument> for Location {
    /// Convert the [LocationDocument] into a [Location].
    fn from(md: LocationDocument) -> Self {
        Self {
            name: format!("{}, {}", md.city, md.state_id),
            city: md.city,
            state_id: md.state_id,
            state: md.state,
            alias: md.alias,
            longitude: md.longitude,
            latitude: md.latitude,
            tz: md.tz,
        }
    }
}
impl From<&Location> for LocationDocument {
    /// Convert the [Location] into a [LocationDocument]
    fn from(location: &Location) -> Self {
        Self::from(location.clone())
    }
}
impl From<Location> for LocationDocument {
    /// Convert the [Location] into a [LocationDocument]
    fn from(location: Location) -> Self {
        Self {
            name: format!("{}, {}", location.city, location.state_id),
            city: location.city,
            state_id: location.state_id,
            state: location.state,
            alias: location.alias,
            longitude: location.longitude,
            latitude: location.latitude,
            tz: location.tz,
            // the location document will always be valid coming from a location
            valid: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::testlib;
    #[test]
    fn location_valid() {
        let mut location = LocationDocument {
            city: "city".to_string(),
            state_id: "abrev_state".to_string(),
            state: "state".to_string(),
            name: "name".to_string(),
            alias: "alias".to_string(),
            latitude: "0".to_string(),
            longitude: "0".to_string(),
            tz: "utc".to_string(),
            valid: false,
        };
        assert!(location.ok(0));
        assert!(location.valid);
        // city
        location.city = Default::default();
        assert!(!location.ok(0));
        assert!(!location.valid);
        location.city = "city".to_string();
        assert!(location.ok(0));
        // short state
        location.state_id = Default::default();
        assert!(!location.ok(0));
        assert!(!location.valid);
        location.state_id = "XX".to_string();
        assert!(location.ok(0));
        // short state
        location.state = Default::default();
        assert!(!location.ok(0));
        assert!(!location.valid);
        location.state = "state".to_string();
        assert!(location.ok(0));
        // alias
        location.alias = Default::default();
        assert!(!location.ok(0));
        assert!(!location.valid);
        location.alias = "alias".to_string();
        assert!(location.ok(0));
        // latitude
        location.latitude = Default::default();
        assert!(!location.ok(0));
        assert!(!location.valid);
        location.latitude = "0".to_string();
        assert!(location.ok(0));
        // longitude
        location.longitude = Default::default();
        assert!(!location.ok(0));
        assert!(!location.valid);
        location.longitude = "0".to_string();
        assert!(location.ok(0));
        // longitude
        location.tz = Default::default();
        assert!(!location.ok(0));
        assert!(!location.valid);
        location.tz = "utc".to_string();
        assert!(location.ok(0));
    }

    #[test]
    fn location_from_to() {
        let testcase = LocationDocument {
            city: "city".to_string(),
            state_id: "abrev_state".to_string(),
            state: "state".to_string(),
            name: "name".to_string(),
            alias: "alias".to_string(),
            latitude: "0".to_string(),
            longitude: "1".to_string(),
            tz: "utc".to_string(),
            valid: false,
        };
        let location = Location::from(testcase);
        assert_eq!(location.city, "city");
        assert_eq!(location.state_id, "abrev_state");
        assert_eq!(location.state, "state");
        assert_eq!(location.name, "city, abrev_state");
        assert_eq!(location.alias, "alias");
        assert_eq!(location.latitude, "0");
        assert_eq!(location.longitude, "1");
        assert_eq!(location.tz, "utc");
        let location_document = LocationDocument::from(&location);
        assert_eq!(location_document.city, location.city);
        assert_eq!(location_document.state_id, location.state_id);
        assert_eq!(location_document.state, location.state);
        assert_eq!(location_document.name, location.name);
        assert_eq!(location_document.alias, location.alias);
        assert_eq!(location_document.latitude, location.latitude);
        assert_eq!(location_document.longitude, location.longitude);
        assert_eq!(location_document.tz, location.tz);
        assert!(location_document.valid);
    }

    #[test]
    fn documents_purify() {
        macro_rules! document {
            ($name: expr, $alias: expr) => {
                LocationDocument {
                    city: "city".to_ascii_lowercase(),
                    state_id: "abrev_state".to_ascii_lowercase(),
                    state: "state".to_ascii_lowercase(),
                    name: $name.to_string(),
                    alias: $alias.to_string(),
                    latitude: "0".to_string(),
                    longitude: "1".to_string(),
                    tz: "UTC".to_string(),
                    valid: false,
                }
            };
        }
        let location_documents = LocationDocuments {
            locations: vec![
                document!("two", "alias"),
                document!("one", "alias"),
                document!("three", ""),
                document!("four", "four"),
            ],
        };
        let testcase = location_documents.validate_and_dedup();
        assert_eq!(testcase.locations.len(), 2);
        assert_eq!(testcase.locations[0].name, "four");
        assert_eq!(testcase.locations[1].name, "two");
    }

    #[test]
    fn locations_file() {
        let fixture = testlib::TestFixture::create();
        let weather_dir = WeatherDir::try_from(fixture.to_string()).unwrap();
        assert!(!LocationsFile::exists(&weather_dir));
        LocationsFile::create(&weather_dir).unwrap();
        assert!(LocationsFile::exists(&weather_dir));
        fixture.copy_resources(&testlib::test_resources().join("filesys").join("locations.json"));
        let locations_file = LocationsFile::open(&weather_dir).unwrap();
        let mut locations: Vec<Location> = locations_file.load().unwrap().map(|location| location.into()).collect();
        assert_eq!(locations.len(), 3);
        assert_eq!(locations[0].name, "Between City, KS");
        assert_eq!(locations[1].name, "Northern City, MT");
        assert_eq!(locations[2].name, "Southern City, GA");
        locations.push(Location {
            city: "City".to_string(),
            state_id: "ST".to_string(),
            state: "State".to_string(),
            name: "City, ST".to_string(),
            alias: "alias".to_string(),
            longitude: "0".to_string(),
            latitude: "1".to_string(),
            tz: "UTC".to_string(),
        });
        let location_documents: Vec<LocationDocument> = locations.iter().map(|location| location.into()).collect();
        locations_file.save(location_documents).unwrap();
        let locations: Vec<Location> = locations_file.load().unwrap().map(|location| location.into()).collect();
        assert_eq!(locations.len(), 4);
        assert_eq!(locations[1].name, "City, ST");
    }
}
