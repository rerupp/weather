//! The data model for weather history locations.
//!
mod locations_file;
mod validate;

use crate::{
    backend::filesys::{history_archive::HistoryArchive, WeatherDir},
    entities::{Location, LocationFilter},
};
use locations_file::{LocationDocument, LocationsFile};

#[doc(hidden)]
/// Create an error from the locations specific error message.
macro_rules! err {
    ($($arg:tt)*) => {
        Err(crate::Error(format!("Locations {}", format!($($arg)*))))
    };
}

/// The file system locations API.
pub struct Locations<'w> {
    /// The locations file API.
    file: LocationsFile,
    /// You need to hang onto the weather dir in order to add a location and create the associated archive.
    weather_dir: &'w WeatherDir,
}
impl<'w> Locations<'w> {
    /// The admin module uses this to test if the locations file exists or not.
    ///
    /// # Arguments
    ///
    /// * `weather_dir` is the parent directory of the locations file.
    ///
    pub fn exists(weather_dir: &WeatherDir) -> bool {
        LocationsFile::exists(weather_dir)
    }

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

    /// Get locations optionally selecting specific ones.
    ///
    /// # Arguments
    ///
    /// * `filters` are used select locations.
    ///
    pub fn get(&self, filters: Option<Vec<LocationFilter>>) -> crate::Result<impl Iterator<Item = Location>> {
        let document_iterator = Box::new(self.file.load()?.into_iter());
        let filters = filters.unwrap_or_else(|| vec![]);
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
        use std::fmt::Write;
        let mut validation_errors = String::new();
        macro_rules! validate {
            ($attr: ident) => {
                match validate::$attr(&location.$attr) {
                    Ok(attr) => location.$attr = attr,
                    Err(problem) => write!(validation_errors, "\n  {problem}").unwrap(),
                }
            };
        }
        validate!(country_name);
        validate!(country_code);
        validate!(region_name);
        validate!(region_code);
        validate!(city_name);
        validate!(alias);
        validate!(latitude);
        validate!(longitude);
        validate!(tz);
        if validation_errors.len() > 0 {
            err!("there are validation error for the new location {location}:{}", validation_errors)?;
        }

        // get the file contents and make sure the alias is unique
        let mut location_documents: Vec<LocationDocument> = self.file.load()?.collect();
        let duplicate = location_documents.iter().find(|document| document.alias == location.alias);
        if let Some(document) = duplicate {
            err!("{document} already uses the alias name")?;
        }

        // make sure the history archive does not exist before saving the location
        let archive_file = self.weather_dir.archive(&location.alias);
        if archive_file.exists() {
            err!("The history archive for {location} already exists.")?;
        }

        // make sure the documents are in location name order before saving
        location_documents.push(LocationDocument::from(&location));
        location_documents.sort();
        self.file.save(location_documents)?;

        // create the archive
        HistoryArchive::create(&location.alias, archive_file)?;
        Ok(location)
    }

    /// Update a locations properties. The resulting location will only contain the attributes
    /// that were updated.
    ///
    /// # Arguments
    ///
    /// * `location` identifies the location and contains the new properties.
    ///
    pub fn update(&self, mut location: Location) -> crate::Result<Option<Location>> {
        // JIC verify the alias
        location.alias = validate::alias(&location.alias)?;
        // get the locations document
        let mut location_documents = self.file.load()?.collect::<Vec<_>>();
        let index = match location_documents.iter().position(|l| l.alias == location.alias) {
            Some(index) => index,
            None => err!("Did not find location {location} to update.")?,
        };
        let document = location_documents.get_mut(index).unwrap();

        // even though it should come in okay, validate JIC
        let mut changed = false;
        use std::fmt::Write;
        let mut validation_errors = String::new();
        macro_rules! update_if_changed {
            ($attr: ident) => {
                // if the location attribute is not empty
                if location.$attr.len() > 0 {
                    // validate the incoming location attribute
                    match validate::$attr(&location.$attr) {
                        Err(problem) => write!(validation_errors, "\n  {problem}").unwrap(),
                        Ok(attr) => {
                            // clear the location attribute if it matches the document
                            if document.$attr == attr {
                                location.$attr.clear();
                            } else {
                                // make sure the document and returned location attributes match
                                document.$attr = attr;
                                location.$attr = document.$attr.clone();
                                changed = true;
                            }
                        }
                    }
                }
            };
        }
        update_if_changed!(country_code);
        update_if_changed!(country_name);
        update_if_changed!(region_code);
        update_if_changed!(region_name);
        update_if_changed!(city_name);
        update_if_changed!(latitude);
        update_if_changed!(longitude);
        update_if_changed!(tz);
        if validation_errors.len() > 0 {
            err!("there are validation error for location {location} update:{}", validation_errors)?;
        }

        if changed {
            self.file.save(location_documents)?;
            Ok(Some(location))
        } else {
            log::debug!("Location update {document} did not have any changes.");
            Ok(None)
        }
    }

    /// Delete the location and archive data from the filesystem.
    ///
    /// * Arguments
    ///
    /// * `alias` is the location that will be deleted.
    ///
    pub fn delete(&self, alias: &str) -> crate::Result<bool> {
        // get the location collection
        let mut original = self.file.load()?.collect::<Vec<_>>();

        // remove the location that matches the alias name
        let mut update = original.extract_if(.., |l| l.alias != alias).collect::<Vec<_>>();
        if original.len() == update.len() {
            log::warn!("The location was not deleted, the alias '{alias}' was not found.");
            return Ok(false);
        }

        // persist the location collection
        update.sort();
        self.file.save(update)?;

        // try to remove the archive
        if let Err(error) = self.weather_dir.archive(alias).remove() {
            // don't fail at this point until you can transact the locations update
            log::error!("Locations failed to delete the '{alias}' archive: {error}");
        }
        Ok(true)
    }
}

/// An iterator that returns locations from a source JSON document. The iterator
/// will optionally filter the results based on a collection of locations filters.
///
struct LocationsIterator {
    /// The iterator that walks the location documents.
    documents: Box<dyn Iterator<Item = LocationDocument>>,
    /// The document filter.
    filters: Vec<LocationFilter>,
}
impl LocationsIterator {
    /// Creates a new instance of the location iterator.
    ///
    /// # Arguments
    ///
    /// * `documents` is the source document location iterator.
    /// * `filters` optionally select which locations will be returned.
    ///
    fn new(documents: Box<dyn Iterator<Item = LocationDocument>>, filters: Vec<LocationFilter>) -> Self {
        Self {
            documents,
            filters: filters
                .into_iter()
                .filter_map(|mut filter| {
                    // ignore empty filters
                    if filter.is_none() {
                        log::debug!("LocationsIterator filter is empty.");
                        None
                    } else {
                        // force the filter patterns to be lowercase
                        if let Some(alias) = filter.alias.take() {
                            filter.alias.replace(alias.to_lowercase());
                        }
                        if let Some(city) = filter.city.take() {
                            filter.city.replace(city.to_lowercase());
                        }
                        if let Some(state) = filter.region.take() {
                            filter.region.replace(state.to_lowercase());
                        }
                        if let Some(name) = filter.country.take() {
                            filter.country.replace(name.to_lowercase());
                        }
                        Some(filter)
                    }
                })
                .collect(),
        }
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
        let mut include_location = false;
        for filter in self.filters.iter() {
            if let Some(alias) = &filter.alias {
                if !is_match(alias, &location.alias) {
                    continue;
                }
            }
            if let Some(city) = &filter.city {
                if !is_match(city, &location.city_name) {
                    continue;
                }
            }
            if let Some(region) = &filter.region {
                if !(is_match(region, &location.region_name) || is_match(region, &location.region_code)) {
                    continue;
                }
            }
            if let Some(country) = &filter.country {
                if !(is_match(country, &location.country_name) || is_match(country, &location.country_code)) {
                    continue;
                }
            }

            // getting here means the filter passed since there are no empty filters
            include_location = true;
            break;
        }
        include_location
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

/// Test if there is a match between some string pattern and a string value.
/// Comparisons are case-insensitive.
///
/// # Arguments
///
/// * `pattern` follows the form of *STRING|STRING*|*STRING*|*|STRING.
/// * `value` is what the pattern will be compared to.
///
fn is_match(pattern: &str, value: &str) -> bool {
    if pattern == "*" {
        true
    } else {
        let pattern = pattern.to_lowercase();
        let value = value.to_lowercase();
        match (pattern.starts_with("*"), pattern.ends_with("*")) {
            (true, true) => value.contains(&pattern[1..pattern.len() - 1]),
            (true, false) => value.ends_with(&pattern[1..]),
            (false, true) => value.starts_with(&pattern[..pattern.len() - 1]),
            _ => value == pattern,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::testlib;

    #[test]
    fn locations() {
        let fixture = testlib::TestFixture::create();
        fixture.copy_resources(&testlib::test_resources().join("filesys").join("locations.json"));
        let weather_dir = WeatherDir::try_from(fixture.to_string()).unwrap();
        let locations = Locations::open(&weather_dir).unwrap();

        let testcase: Vec<Location> = locations.get(None).unwrap().collect();
        assert_eq!(testcase.len(), 3);
        assert_eq!(testcase[0].alias, "between");
        assert_eq!(testcase[1].alias, "north");
        assert_eq!(testcase[2].alias, "south");

        let testcase = locations.get(Some(vec![LocationFilter::alias("*tH")])).unwrap().collect::<Vec<Location>>();
        assert_eq!(testcase.len(), 2);
        assert_eq!(testcase[0].alias, "north");
        assert_eq!(testcase[1].alias, "south");

        let testcase = locations
            .get(Some(vec![LocationFilter::region("ga"), LocationFilter::region("mt")]))
            .unwrap()
            .collect::<Vec<Location>>();
        assert_eq!(testcase.len(), 2);
        assert_eq!(testcase[0].alias, "north");
        assert_eq!(testcase[1].alias, "south");

        let location = Location {
            country_name: "Country".to_string(),
            country_code: "CO".to_string(),
            region_name: "Region ".to_string(),
            region_code: " RN".to_string(),
            city_name: "  New City".to_string(),
            alias: " nEw ".to_string(),
            latitude: "1 ".to_string(),
            longitude: " 0 ".to_string(),
            tz: "utc".to_string(),
        };
        let location = locations.add(location).unwrap();
        assert_eq!(location.country_name, "Country");
        assert_eq!(location.country_code, "CO");
        assert_eq!(location.region_name, "Region");
        assert_eq!(location.region_code, "RN");
        assert_eq!(location.city_name, "New City");
        assert_eq!(location.alias, "new");
        assert_eq!(location.latitude, "1");
        assert_eq!(location.longitude, "0");
        assert_eq!(location.tz, "UTC");
        let testcase: Vec<Location> = locations.get(None).unwrap().collect();
        assert_eq!(testcase.len(), 4);
        assert!(testcase.iter().find(|location| &location.alias == "new").is_some());
        assert!(weather_dir.archive(&location.alias).exists());

        // update the new location
        let update = Location {
            country_name: " updated Country".to_string(),
            country_code: "updated CO".to_string(),
            region_name: " updated Region".to_string(),
            region_code: "updated RN".to_string(),
            city_name: "Updated City ".to_string(),
            alias: "new".to_string(),
            latitude: " -1".to_string(),
            longitude: "1 ".to_string(),
            tz: "america/phoenix".to_string(),
        };
        // unwrap the result and get the option value
        let updated_location = locations.update(update).unwrap().unwrap();
        assert_eq!(updated_location.country_name, "updated Country");
        assert_eq!(updated_location.country_code, "updated CO");
        assert_eq!(updated_location.region_name, "updated Region");
        assert_eq!(updated_location.region_code, "updated RN");
        assert_eq!(updated_location.city_name, "Updated City");
        assert_eq!(updated_location.alias, "new");
        assert_eq!(updated_location.latitude, "-1");
        assert_eq!(updated_location.longitude, "1");
        assert_eq!(updated_location.tz, "America/Phoenix");

        // check partial update
        let partial_update = Location {
            country_name: "Partial Update Country".to_string(),
            country_code: "Partial Update CO".to_string(),
            region_name: "State".to_string(),
            region_code: "id".to_string(),
            city_name: "Partial Update City".to_string(),
            alias: "new".to_string(),
            latitude: "".to_string(),
            longitude: "".to_string(),
            tz: "".to_string(),
        };
        assert!(locations.update(partial_update).unwrap().is_some());

        // verify the document contents
        let mut documents = locations.get(Some(vec![LocationFilter::alias("new")])).unwrap().collect::<Vec<_>>();
        assert_eq!(documents.len(), 1);
        let testcase = documents.pop().unwrap();
        assert_eq!(testcase.country_name, "Partial Update Country");
        assert_eq!(testcase.country_code, "Partial Update CO");
        assert_eq!(testcase.region_name, "State");
        assert_eq!(testcase.region_code, "id");
        assert_eq!(testcase.city_name, "Partial Update City");
        assert_eq!(testcase.alias, updated_location.alias);
        assert_eq!(testcase.latitude, updated_location.latitude);
        assert_eq!(testcase.longitude, updated_location.longitude);
        assert_eq!(testcase.tz, updated_location.tz);

        // delete the new location
        locations.delete("new").unwrap();
        assert!(locations.get(None).unwrap().find(|l| l.alias == "new").is_none());
    }

    #[test]
    fn iterator() {
        let fixture = testlib::TestFixture::create();
        fixture.copy_resources(&testlib::test_resources().join("filesys").join("locations.json"));
        let weather_dir = WeatherDir::try_from(fixture.to_string()).unwrap();
        let locations_file = LocationsFile::open(&weather_dir).unwrap();

        macro_rules! testcase {
            ($filters:expr) => {
                LocationsIterator::new(Box::new(locations_file.load().unwrap()), $filters).collect::<Vec<_>>()
            };
        }
        assert_eq!(testcase!(vec![]).len(), 3);
        assert_eq!(testcase!(vec![LocationFilter::city("Southern City")]).len(), 1);
        assert_eq!(testcase!(vec![LocationFilter::alias("south")]).len(), 1);
        assert_eq!(testcase!(vec![LocationFilter::region("Kansas")]).len(), 1);
        assert_eq!(testcase!(vec![LocationFilter::region("KS")]).len(), 1);
        assert_eq!(testcase!(vec![LocationFilter::country("United States")]).len(), 3);
        assert_eq!(testcase!(vec![LocationFilter::country("us")]).len(), 3);
        assert_eq!(testcase!(vec![LocationFilter::city("South*").with_region("GA")]).len(), 1);
        assert_eq!(testcase!(vec![LocationFilter::city("South*").with_country("US")]).len(), 1);
        assert_eq!(testcase!(vec![LocationFilter::region("GA").with_country("us")]).len(), 1);

        let locations = testcase!(vec![LocationFilter::city("south*").with_region("ga").with_country("us")]);
        assert_eq!(locations.len(), 1);

        let locations = testcase!(vec![
            LocationFilter::city("Southern City"),
            LocationFilter::city("between City"),
            LocationFilter::alias("north"),
        ]);
        assert_eq!(locations.len(), 3);
    }

    #[test]
    fn is_match() {
        assert!(super::is_match("*", ""));
        assert!(super::is_match("*", "value"));
        assert!(super::is_match("*ue", "valUE"));
        assert!(!super::is_match("ue", "valUE"));
        assert!(super::is_match("va*", "Value"));
        assert!(!super::is_match("va", "Value"));
        assert!(super::is_match("*al*", "vALue"));
        assert!(!super::is_match("al", "vALue"));
        assert!(super::is_match("value", "VALUE"));
    }
}
