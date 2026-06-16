//! The manager of a location document in the filesystem.
use super::validate;
use crate::{
    backend::filesys::{WeatherDir, WeatherFile},
    entities::{location_equal, location_order, Location},
};
use serde::{Deserialize, Serialize};
use std::io::{BufWriter, Write};

/// The name of the locations document in the weather data directory.
const LOCATIONS_FILENAME: &'static str = "locations.json";

/// The name of the updated locations document in the weather data directory.
const UPDATE_EXTENSION: &'static str = "upd";

/// The name of the backup locations document in the weather data directory.
const BACKUP_EXTENSION: &'static str = "bck";

#[doc(hidden)]
/// Create an Err for the error.
macro_rules! err {
    ($($args:tt)*) => {
        Err(crate::Error(format!("LocationsFile {}", format!($($args)*))))
    }
}

/// The locations `JSON` document manager.
#[derive(Debug)]
pub struct LocationsFile {
    /// The `JSON` document file.
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
            if location.ok(index) {
                locations.push(location);
            }
        }
        // order the documents by alias to remove duplicates
        locations.sort_by(|lhs, rhs| lhs.alias.cmp(&rhs.alias));
        locations.dedup_by(|lhs, rhs| {
            let duplicate = lhs.alias == rhs.alias;
            if duplicate {
                log::warn!("The alias name is being used by {rhs}.");
            }
            duplicate
        });
        // finally order the documents by name
        locations.sort();
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

/// The bean that describes the metadata for a location.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct LocationDocument {
    /// A unique nickname of a location.
    pub alias: String,
    /// The location country name such as *United States* or *Canada*.
    #[serde(rename = "country-name")]
    pub country_name: String,
    /// The location country code such as *US* or *CA*.
    #[serde(rename = "country-code")]
    pub country_code: String,
    /// The location region name such as *Arizona* or *British Columbia*.
    #[serde(rename = "region-name")]
    pub region_name: String,
    /// The location region code such as *AZ* or *BC*.
    #[serde(rename = "region-code")]
    pub region_code: String,
    /// The location city name.
    #[serde(rename = "city-name")]
    pub city_name: String,
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
impl std::fmt::Display for LocationDocument {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}, {} ({})", self.city_name, self.region_code, self.alias)
    }
}
impl From<LocationDocument> for Location {
    /// Convert the [LocationDocument] into a [Location].
    fn from(document: LocationDocument) -> Self {
        Self {
            country_name: document.country_name,
            country_code: document.country_code,
            region_name: document.region_name,
            region_code: document.region_code,
            city_name: document.city_name,
            alias: document.alias,
            longitude: document.longitude,
            latitude: document.latitude,
            tz: document.tz,
        }
    }
}
impl From<&LocationDocument> for Location {
    fn from(document: &LocationDocument) -> Self {
        document.clone().into()
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
            country_name: location.country_name,
            country_code: location.country_code,
            region_name: location.region_name,
            region_code: location.region_code,
            city_name: location.city_name,
            alias: location.alias,
            longitude: location.longitude,
            latitude: location.latitude,
            tz: location.tz,
            // the location document will always be valid coming from a location
            valid: true,
        }
    }
}
impl Ord for LocationDocument {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        location_order!(self, other)
    }
}
impl PartialOrd for LocationDocument {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for LocationDocument {
    fn eq(&self, other: &Self) -> bool {
        location_equal!(self, other)
    }
}
impl Eq for LocationDocument {}
impl LocationDocument {
    /// Verify the location document is valid.
    ///
    /// Arguments
    ///
    /// * `index` is the position of the location within the JSON document.
    ///
    pub fn ok(&mut self, index: usize) -> bool {
        use std::fmt::Write;

        // set up capturing validation failures.
        self.valid = true;
        let mut validation_failures = String::default();
        macro_rules! add_failure {
            ($failure: expr) => {
                write!(validation_failures, "\n  {}", $failure).unwrap()
            };
        }
        match validate::city_name(&self.city_name) {
            Ok(name) => self.city_name = name,
            Err(failure) => add_failure!(failure),
        }
        match validate::region_code(&self.region_code) {
            Ok(code) => self.region_code = code,
            Err(failure) => add_failure!(failure),
        }
        match validate::region_name(&self.region_name) {
            Ok(name) => self.region_name = name,
            Err(failure) => add_failure!(failure),
        }
        match validate::country_code(&self.country_code) {
            Ok(code) => self.country_code = code,
            Err(failure) => add_failure!(failure),
        }
        match validate::country_name(&self.country_name) {
            Ok(name) => self.country_name = name,
            Err(failure) => add_failure!(failure),
        }
        match validate::alias(&self.alias) {
            Ok(alias) => self.alias = alias,
            Err(failure) => add_failure!(failure),
        }
        match validate::latitude(&self.latitude) {
            Ok(latitude) => self.latitude = latitude,
            Err(failure) => add_failure!(failure),
        }
        match validate::longitude(&self.longitude) {
            Ok(longitude) => self.longitude = longitude,
            Err(failure) => add_failure!(failure),
        }
        match validate::tz(&self.tz) {
            Ok(tz) => self.tz = tz,
            Err(failure) => add_failure!(failure),
        }
        if !validation_failures.is_empty() {
            log::warn!("Location at document index {index} did not validate:{}", validation_failures);
            self.valid = false;
        }
        self.valid
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::testlib;

    #[test]
    fn location_valid() {
        let mut document = LocationDocument {
            country_name: "Country Name".to_string(),
            country_code: "CN".to_string(),
            region_name: "Region Name".to_string(),
            region_code: "RC".to_string(),
            city_name: "City Name".to_string(),
            alias: "alias".to_string(),
            latitude: "0".to_string(),
            longitude: "0".to_string(),
            tz: "utc".to_string(),
            valid: false,
        };
        assert!(document.ok(0));
        assert!(document.valid);

        macro_rules! test_validation {
            ($attr:ident) => {
                let before_check = document.$attr;
                document.$attr = Default::default();
                assert!(!document.ok(0));
                assert!(!document.valid);
                document.$attr = before_check;
                assert!(document.ok(0));
                assert!(document.valid);
            };
        }
        test_validation!(country_name);
        test_validation!(country_code);
        test_validation!(region_name);
        test_validation!(region_code);
        test_validation!(city_name);
        test_validation!(alias);
        test_validation!(latitude);
        test_validation!(longitude);
        test_validation!(tz);
    }

    #[test]
    fn location_from_to() {
        let location_document = LocationDocument {
            country_name: "Country Name".to_string(),
            country_code: "CN".to_string(),
            region_name: "Region Name".to_string(),
            region_code: "RN".to_string(),
            city_name: "City Name".to_string(),
            alias: "alias".to_string(),
            latitude: "0".to_string(),
            longitude: "1".to_string(),
            tz: "utc".to_string(),
            valid: false,
        };
        let location = Location::from(location_document);
        assert_eq!(location.country_name, "Country Name");
        assert_eq!(location.country_code, "CN");
        assert_eq!(location.region_name, "Region Name");
        assert_eq!(location.region_code, "RN");
        assert_eq!(location.city_name, "City Name");
        assert_eq!(location.alias, "alias");
        assert_eq!(location.latitude, "0");
        assert_eq!(location.longitude, "1");
        assert_eq!(location.tz, "utc");

        // round trip the document from the location
        let location_document = LocationDocument::from(&location);
        assert_eq!(location_document.country_name, location.country_name);
        assert_eq!(location_document.country_code, location.country_code);
        assert_eq!(location_document.region_name, location.region_name);
        assert_eq!(location_document.region_code, location.region_code);
        assert_eq!(location_document.city_name, location.city_name);
        assert_eq!(location_document.alias, location.alias);
        assert_eq!(location_document.latitude, location.latitude);
        assert_eq!(location_document.longitude, location.longitude);
        assert_eq!(location_document.tz, location.tz);
        assert!(location_document.valid);
    }

    #[test]
    fn documents_purify() {
        macro_rules! document {
            ($city: expr, $alias: expr) => {
                LocationDocument {
                    country_name: "Country Name".to_string(),
                    country_code: "CN".to_string(),
                    region_name: "Region Name".to_string(),
                    region_code: "RN".to_string(),
                    city_name: $city.to_string(),
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
        assert_eq!(testcase.locations[0].city_name, "four");
        assert_eq!(testcase.locations[1].city_name, "two");
    }

    #[test]
    fn locations_file() {
        let fixture = testlib::TestFixture::create();
        let weather_dir = WeatherDir::try_from(fixture.to_string()).unwrap();

        // create the location document
        assert!(!LocationsFile::exists(&weather_dir));
        LocationsFile::create(&weather_dir).unwrap();
        assert!(LocationsFile::exists(&weather_dir));

        // copy the test resource and load it
        fixture.copy_resources(&testlib::test_resources().join("filesys").join("locations.json"));
        let locations_file = LocationsFile::open(&weather_dir).unwrap();
        let mut locations: Vec<Location> = locations_file.load().unwrap().map(|location| location.into()).collect();
        assert_eq!(locations.len(), 3);
        assert_eq!(locations[0].city_name, "Between City");
        assert_eq!(locations[1].city_name, "Northern City");
        assert_eq!(locations[2].city_name, "Southern City");

        // add a new location
        locations.push(Location {
            country_name: "United States".to_string(),
            country_code: "US".to_string(),
            region_name: "Oregon".to_string(),
            region_code: "OR".to_string(),
            city_name: "King City".to_string(),
            alias: "alias".to_string(),
            latitude: "45.4012".to_string(),
            longitude: "-122.8069".to_string(),
            tz: "America/Los_Angeles".to_string(),
        });
        let location_documents: Vec<LocationDocument> = locations.iter().map(|location| location.into()).collect();
        locations_file.save(location_documents).unwrap();

        // verify the new location is there
        let locations: Vec<Location> = locations_file.load().unwrap().map(|location| location.into()).collect();
        assert_eq!(locations.len(), 4);
        assert_eq!(locations[0].city_name, "Between City");
        assert_eq!(locations[1].city_name, "King City");
        assert_eq!(locations[2].city_name, "Northern City");
        assert_eq!(locations[3].city_name, "Southern City");
    }
}
