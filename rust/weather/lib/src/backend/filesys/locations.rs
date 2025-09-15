//! The data model for weather data locations.
//!
mod locations_file;
mod validate;

use crate::{
    backend::filesys::{HistoryArchive, WeatherDir},
    entities::{Location, LocationFilters},
    location_filters,
};
use locations_file::{LocationDocument, LocationsFile};

/// Create a Locations specific error message.
macro_rules! error {
    ($($arg:tt)*) => {
        crate::Error::from(format!("Locations {}", format!($($arg)*)))
    }
}

/// Create an error from the locations specific error message.
macro_rules! err {
    ($($arg:tt)*) => {
        Err(error!($($arg)*))
    };
}

/// The file system locations API.
pub struct Locations<'w> {
    /// The locations file API.
    file: LocationsFile,
    /// You need to hang onto the weather dir in order to add a location and create the associated
    /// archive.
    weather_dir: &'w WeatherDir,
}
impl<'w> Locations<'w> {
    /// Opens an existing locations file or create a new one if it does not exist.
    ///
    /// # Arguments
    ///
    /// * `weather_dir` is the location of the locations file.
    ///
    pub fn open(weather_dir: &'w WeatherDir) -> crate::Result<Self> {
        let file = match LocationsFile::exists(weather_dir) {
            true => LocationsFile::open(weather_dir)?,
            false => LocationsFile::create(weather_dir)?,
        };
        Ok(Self { file, weather_dir })
    }

    /// Get all locations.
    ///
    pub fn get(&self) -> crate::Result<impl Iterator<Item = Location>> {
        let document_iterator = Box::new(self.file.load()?.into_iter());
        Ok(LocationsIterator::new(document_iterator, location_filters![]))
    }

    /// Get locations based on a collection of selection filters.
    ///
    /// # Arguments
    ///
    /// * `filters` are used select locations.
    ///
    pub fn find(&self, filters: LocationFilters) -> crate::Result<impl Iterator<Item = Location>> {
        let document_iterator = Box::new(self.file.load()?.into_iter());
        Ok(LocationsIterator::new(document_iterator, filters))
    }

    /// Add a location to the locations document.
    ///
    /// # Arguments
    ///
    /// * `location` is the location that will be added.
    ///
    pub fn add(&self, mut location: Location) -> crate::Result<Location> {
        // even though it should come in okay, validate JIC
        location.city = validate::city(&location.city)?;
        location.state_id = validate::city(&location.state_id)?;
        location.state = validate::city(&location.state)?;
        // todo: this can go away once not persisting
        location.name = validate::name(&location.name)?;
        location.alias = validate::alias(&location.alias)?;
        location.latitude = validate::latitude(&location.latitude)?;
        location.longitude = validate::longitude(&location.longitude)?;
        location.tz = validate::tz(&location.tz)?;

        // get the file contents and make sure the alias is unique
        let mut location_documents: Vec<LocationDocument> = self.file.load()?.collect();
        let found_alias = location_documents.iter().find(|location_document| location_document.alias == location.alias);
        if let Some(location_document) = found_alias {
            err!("{} already uses the '{}' alias name", location_document.name, location_document.alias)?;
        }

        // make sure the documents are in location name order before saving
        location_documents.push(LocationDocument::from(&location));
        // todo: change to city and state?
        location_documents.sort_by(|lhs, rhs| lhs.name.cmp(&rhs.name));
        self.file.save(location_documents)?;
        let archive = self.weather_dir.archive(&location.alias);
        HistoryArchive::create(&location.alias, archive)?;
        Ok(location)
    }
}

/// An iterator that returns locations from a source JSON document. The iterator
/// will optionally filter the results based on a collection of locations filters.
///
struct LocationsIterator {
    /// The iterator that walks the location documents.
    documents: Box<dyn Iterator<Item = LocationDocument>>,

    /// The document filter.
    filters: LocationFilters,
}
impl LocationsIterator {
    /// Creates a new instance of the location iterator.
    ///
    /// # Arguments
    ///
    /// * `documents` is the source document location iterator.
    /// * `filters` optionally select which locations will be returned..
    ///
    fn new(documents: Box<dyn Iterator<Item = LocationDocument>>, mut filters: LocationFilters) -> Self {
        // force all the filter pattern to be lowercase
        for filter in filters.iter_mut() {
            if let Some(city) = filter.city.take() {
                filter.city.replace(city.to_lowercase());
            }
            if let Some(state) = filter.state.take() {
                filter.state.replace(state.to_lowercase());
            }
            if let Some(name) = filter.name.take() {
                filter.name.replace(name.to_lowercase());
            }
        }
        Self { documents, filters }
    }

    /// Determine if a location should be returned from the iterator.
    ///
    /// # Arguments
    ///
    /// * `location` is the document that will be inspected.
    ///
    fn include(&self, location: &LocationDocument) -> bool {
        // it's a no-brainer if there are no filters
        if self.filters.is_empty() {
            return true;
        }

        // loop through the filters to find a match
        for filter in self.filters.iter() {
            match (&filter.city, &filter.state, &filter.name) {
                (Some(city), None, None) => {
                    if Self::is_match(city, &location.city) {
                        return true;
                    }
                }
                (None, Some(state), None) => {
                    if Self::is_state_match(state, location) {
                        return true;
                    };
                }
                (None, None, Some(name)) => {
                    if Self::is_name_match(name, location) {
                        return true;
                    };
                }
                (Some(city), Some(state), None) => {
                    if Self::is_match(city, &location.city) && Self::is_state_match(state, location) {
                        return true;
                    };
                }
                (Some(city), None, Some(name)) => {
                    if Self::is_match(city, &location.city) && Self::is_name_match(name, location) {
                        return true;
                    }
                }
                (None, Some(state), Some(name)) => {
                    if Self::is_state_match(state, location) && Self::is_name_match(name, location) {
                        return true;
                    }
                }
                (Some(city), Some(state), Some(name)) => {
                    if Self::is_match(city, &location.city)
                        && Self::is_state_match(state, location)
                        && Self::is_name_match(name, location)
                    {
                        return true;
                    }
                }
                _ => (),
            }
        }
        false
    }

    /// Test if there is a match with the location name or alias.
    ///
    /// # Arguments
    ///
    /// * `pattern` follows the form of *STRING|STRING*|*STRING*|*|STRING.
    /// * `location` is the location document that will be tested.
    ///
    fn is_name_match(pattern: &str, location: &LocationDocument) -> bool {
        Self::is_match(pattern, &location.alias) || Self::is_match(pattern, &location.name)
    }

    /// Test if there is a match with the location state name or two-letter abbreviation.
    ///
    /// # Arguments
    ///
    /// * `pattern` follows the form of *STRING|STRING*|*STRING*|*|STRING.
    /// * `location` is the location document that will be tested.
    ///
    fn is_state_match(pattern: &str, location: &LocationDocument) -> bool {
        if pattern.len() > 2 {
            if Self::is_match(pattern, &location.state) {
                return true;
            }
        }
        if Self::is_match(pattern, &location.state_id) {
            return true;
        }
        false
    }

    /// Test if there is a match between some string pattern and a string value.
    /// Comparisons are case-insensitive.
    ///
    /// # Arguments
    ///
    /// * `pattern` follows the form of *STRING|STRING*|*STRING*|*|STRING.
    /// * `value` is what the pattern will be compared to.
    ///
    fn is_match(pattern: &str, value: &str) -> bool {
        let value = value.to_lowercase();
        if pattern == "*" {
            return true;
        }
        if pattern.starts_with("*") && pattern.ends_with("*") {
            if value.contains(&pattern[1..pattern.len() - 1]) {
                return true;
            }
        }
        if pattern.starts_with("*") {
            if value.ends_with(&pattern[1..]) {
                return true;
            }
        }
        if pattern.ends_with("*") {
            if value.starts_with(&pattern[..pattern.len() - 1]) {
                return true;
            }
        }
        if pattern == value {
            return true;
        }
        false
    }
}
impl Iterator for LocationsIterator {
    type Item = Location;
    fn next(&mut self) -> Option<Self::Item> {
        let mut next_location = None;
        loop {
            match self.documents.next() {
                None => break,
                Some(document) => {
                    if self.include(&document) {
                        next_location.replace(document.into());
                        break;
                    }
                }
            }
        }
        next_location
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{backend::testlib, location_filter};

    #[test]
    fn locations() {
        let fixture = testlib::TestFixture::create();
        fixture.copy_resources(&testlib::test_resources().join("filesys").join("locations.json"));
        let weather_dir = WeatherDir::try_from(fixture.to_string()).unwrap();
        let locations = Locations::open(&weather_dir).unwrap();

        let testcase: Vec<Location> = locations.get().unwrap().collect();
        assert_eq!(testcase.len(), 3);
        assert_eq!(testcase[0].alias, "between");
        assert_eq!(testcase[1].alias, "north");
        assert_eq!(testcase[2].alias, "south");

        let testcase =
            locations.find(location_filters![location_filter!(name = "*tH")]).unwrap().collect::<Vec<Location>>();
        assert_eq!(testcase.len(), 2);
        assert_eq!(testcase[0].alias, "north");
        assert_eq!(testcase[1].alias, "south");

        let testcase = locations
            .find(location_filters![location_filter!(name = "north"), location_filter!(name = "south"),])
            .unwrap()
            .collect::<Vec<Location>>();
        assert_eq!(testcase.len(), 2);
        assert_eq!(testcase[0].alias, "north");
        assert_eq!(testcase[1].alias, "south");

        let location = Location {
            city: "New City".to_string(),
            state_id: "abrev_state".to_string(),
            state: "state".to_string(),
            name: " New City".to_string(),
            alias: "nEw".to_string(),
            latitude: "1 ".to_string(),
            longitude: " 0 ".to_string(),
            tz: "utc".to_string(),
        };
        let location = locations.add(location).unwrap();
        assert_eq!(location.name, "New City");
        assert_eq!(location.alias, "new");
        assert_eq!(location.latitude, "1");
        assert_eq!(location.longitude, "0");
        assert_eq!(location.tz, "UTC");
        let testcase: Vec<Location> = locations.get().unwrap().collect();
        assert_eq!(testcase.len(), 4);
        assert!(testcase.iter().find(|location| &location.alias == "new").is_some());
        assert!(weather_dir.archive(&location.alias).exists());

        let testcase: Vec<Location> =
            locations.find(location_filters![location_filter!(name = "new")]).unwrap().collect();
        assert_eq!(testcase.len(), 1);
    }

    #[test]
    fn iterator_matching() {
        assert!(LocationsIterator::is_match("*", "value"));
        assert!(LocationsIterator::is_match("*ue", "valUE"));
        assert!(LocationsIterator::is_match("v*", "Value"));
        assert!(LocationsIterator::is_match("*al*", "vALue"));
        assert!(!LocationsIterator::is_match("al", "vALue"));

        let testcase = LocationDocument::from(Location {
            city: String::from("City"),
            state: String::from("State"),
            state_id: String::from("ST"),
            name: String::from("City, ST"),
            alias: String::from("city"),
            latitude: String::from("1"),
            longitude: String::from("1"),
            tz: String::from("UTC"),
        });
        assert!(LocationsIterator::is_name_match("ci*", &testcase));
        assert!(LocationsIterator::is_name_match("*ty", &testcase));
        assert!(LocationsIterator::is_name_match("*, st", &testcase));
        assert!(LocationsIterator::is_name_match("city", &testcase));
        assert!(LocationsIterator::is_name_match("city, st", &testcase));

        assert!(LocationsIterator::is_state_match("s*", &testcase));
        assert!(LocationsIterator::is_state_match("st*", &testcase));
        assert!(LocationsIterator::is_state_match("sta*", &testcase));
        assert!(LocationsIterator::is_state_match("state", &testcase));
        assert!(LocationsIterator::is_state_match("st", &testcase));
    }

    #[test]
    fn iterator2() {
        let fixture = testlib::TestFixture::create();
        fixture.copy_resources(&testlib::test_resources().join("filesys").join("locations.json"));
        let weather_dir = WeatherDir::try_from(fixture.to_string()).unwrap();
        let locations_file = LocationsFile::open(&weather_dir).unwrap();

        macro_rules! testcase {
            ($filters:expr) => {
                LocationsIterator::new(Box::new(locations_file.load().unwrap()), $filters).collect::<Vec<_>>()
            };
        }
        assert_eq!(testcase!(location_filters!()).len(), 3);
        assert_eq!(testcase!(location_filters![location_filter!(city = "South*")]).len(), 1);
        assert_eq!(testcase!(location_filters![location_filter!(state = "KS")]).len(), 1);
        assert_eq!(testcase!(location_filters![location_filter!(name = "north*")]).len(), 1);
        assert_eq!(testcase!(location_filters![location_filter!(city = "South*", state = "GA")]).len(), 1);
        assert_eq!(testcase!(location_filters![location_filter!(city = "South*").with_name("south")]).len(), 1);
        assert_eq!(testcase!(location_filters![location_filter!(state = "GA").with_name("south")]).len(), 1);

        let locations =
            testcase!(location_filters![location_filter!(city = "Southern City", state = "GA").with_name("south")]);
        assert_eq!(locations.len(), 1);

        let locations = testcase!(location_filters![
            location_filter!(city = "Southern City"),
            location_filter!(state = "KS"),
            location_filter!(name = "north"),
        ]);
        assert_eq!(locations.len(), 3);
    }
}
