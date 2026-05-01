//! Encapsulates reading [simple maps](https://simplemaps.com) city CSV databases.
//!
use super::{CityMD, CountryMD, RegionMD};
use csv::Reader;
use std::{cmp::Ordering, collections::HashSet, path::PathBuf};

/// Create a loader specific error message.
/// 
/// # Params
///
/// * `args` are passed to `format!` to create the error message.
/// 
macro_rules! err {
    ($($args:tt)*) => {
        Err(crate::Error(format!("simple_maps {}", format!($($args)*))))
    };
}

/// Create the Cities components from a Simple Maps CSV file.
/// 
/// # Arguments
/// 
/// * `path` identifies the CSV file that will be parsed.
/// 
pub fn parse(path: &PathBuf) -> crate::Result<(CountryMD, Vec<RegionMD>, Vec<CityMD>)> {
    // create the CSV reader
    if !path.exists() {
        err!("'{}' was not found.", path.display())?;
    }
    if !path.is_file() {
        err!("'{}' is not a file.", path.display())?;
    }
    let mut reader = match Reader::from_path(path) {
        Err(error) => err!("failed to get CSV reader for '{}': {error:?}", path.display()),
        Ok(reader) => Ok(reader),
    }?;

    macro_rules! country {
        ($name: expr, $code: expr) => {
            CountryMD { name: $name.trim().to_string(), code: $code.trim().to_string() }
        };
    }
    // figure out which country you are loading
    let (country, column_map) = match reader.headers() {
        Err(error) => err!("did not get CSV headers: {error:?}"),
        Ok(headers) => match headers.get(2) {
            None => err!("did not get the header to detect which database"),
            Some(header) => match header {
                "state_id" => Ok((country!("United States", "US"), vec![1usize, 3, 2, 6, 7, 13, 15])),
                "province_id" => Ok((country!("Canada", "CA"), vec![1usize, 3, 2, 4, 5, 8, 10])),
                _ => err!("failed to create column map for {header}"),
            },
        },
    }?;

    // read the file creating the Cities metadata
    let mut regions = HashSet::new();
    let mut cities = Vec::new();
    for next_result in reader.into_records() {
        match next_result {
            Err(error) => err!("error reading CSV record ({error:?}).")?,
            // RustRover is braindead understanding the macro expansion consumes the record
            Ok(record) => {
                macro_rules! extract {
                    ($col:expr) => {
                        record.get($col).map_or(Default::default(), |value| value.trim().to_string())
                    };
                }
                // create the city
                let city = CityMD {
                    name: extract!(column_map[0]),
                    region: RegionMD { name: extract!(column_map[1]), code: extract!(column_map[2]) },
                    latitude: extract!(column_map[3]),
                    longitude: extract!(column_map[4]),
                    timezone: extract!(column_map[5]),
                };
                // remember the cities region
                if !regions.contains(&city.region) {
                    regions.insert(city.region.clone());
                }
                cities.push(city);
            }
        }
    }
    let mut regions = regions.into_iter().collect::<Vec<_>>();
    regions.sort_unstable();
    cities.sort_unstable_by(|lhs, rhs| match lhs.name.cmp(&rhs.name) {
        Ordering::Equal => lhs.region.cmp(&rhs.region),
        ordering => ordering,
    });
    Ok((country, regions, cities))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::testlib;

    #[test]
    fn parse() {
        let fixture = testlib::TestFixture::create();
        fixture.copy_resources(&testlib::test_resources().join("cities"));
        let path = PathBuf::from(&fixture);

        // parse the US database
        let (country, regions, cities) = super::parse(&path.join("uscities.csv")).unwrap();
        // verify the country
        assert_eq!(country, CountryMD { name: "United States".to_string(), code: "US".to_string() });
        // verify the regions
        assert_eq!(regions.len(), 8);
        let city_regions = cities.iter().map(|city| city.region.clone()).collect::<HashSet<_>>();
        for region in &regions {
            assert!(city_regions.contains(region), "{region:#?} was not found in US cities region");
        }
        // spot check the cities
        assert_eq!(cities.len(), 9);
        let city = &cities[6];
        assert_eq!(city.name, "New York");
        assert_eq!(city.region.name, "New York");
        assert_eq!(city.region.code, "NY");
        assert!(regions.iter().any(|region| region.eq(&city.region)));
        assert_eq!(city.latitude, "40.6943");
        assert_eq!(city.longitude, "-73.9249");
        assert_eq!(city.timezone, "America/New_York");

        // Canadian
        let (country, regions, cities) = super::parse(&path.join("canadacities.csv")).unwrap();
        assert_eq!(country, CountryMD { name: "Canada".to_string(), code: "CA".to_string() });
        // spot check the regions
        assert_eq!(regions.len(), 5);
        let city_regions = cities.iter().map(|city| city.region.clone()).collect::<HashSet<_>>();
        for region in &regions {
            assert!(city_regions.contains(region), "{region:#?} was not found in CA cities region");
        }
        // spot check the cities
        assert_eq!(cities.len(), 9);
        let toronto = &cities[6];
        assert_eq!(toronto.name, "Toronto");
        assert_eq!(toronto.region.name, "Ontario");
        assert_eq!(toronto.region.code, "ON");
        assert!(regions.iter().any(|region| region.eq(&toronto.region)));
        assert_eq!(toronto.latitude, "43.7417");
        assert_eq!(toronto.longitude, "-79.3733");
        assert_eq!(toronto.timezone, "America/Toronto");
    }
}
