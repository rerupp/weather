/// Creates an error when validation fails.
macro_rules! validate_err {
    ($($arg:tt)*) => {
        Err(crate::Error::from(format!("Validation error: {}", format!($($arg)*))))
    };
}

/// The common boilerplate to all location validation.
macro_rules! validate_not_empty {
    ($what: literal, $value: expr) => {{
        let value = $value.trim();
        if value.is_empty() {
            validate_err!("{} cannot be empty.", $what)?;
        }
        value.to_string()
    }};
}

/// Validate a locations country name.
///
/// # Arguments
///
/// * `name` is the location country name.
///
pub fn country_name(name: &str) -> crate::Result<String> {
    Ok(validate_not_empty!("country name", name))
}

/// Validate a locations country code.
///
/// # Arguments
///
/// * `code` is the location country code.
///
pub fn country_code(code: &str) -> crate::Result<String> {
    Ok(validate_not_empty!("country code", code))
}

/// Validate a locations region name.
///
/// # Arguments
///
/// * `name` is the location region name.
///
pub fn region_name(name: &str) -> crate::Result<String> {
    Ok(validate_not_empty!("region name", name))
}

/// Validate a locations region code.
///
/// # Arguments
///
/// * `code` is the location region code.
///
pub fn region_code(code: &str) -> crate::Result<String> {
    // let state_id = validate_not_empty!("state ID", region_code);
    // Ok(state_id)
    Ok(validate_not_empty!("region code", code))
}

/// Validate a locations city name.
///
/// # Arguments
///
/// * `name` is the name of the city.
///
pub fn city_name(name: &str) -> crate::Result<String> {
    Ok(validate_not_empty!("city name", name))
}

/// Validate a locations alias name.
///
/// # Arguments
///
/// * `alias` is the locations alias name.
///
pub fn alias(alias: &str) -> crate::Result<String> {
    Ok(validate_not_empty!("alias", alias).to_lowercase())
}

/// Validate a locations latitude.
///
/// # Arguments
///
/// * `latitude` is the location latitude.
///
pub fn latitude(latitude: &str) -> crate::Result<String> {
    let latitude = validate_not_empty!("latitude", latitude);
    match latitude.parse::<f64>() {
        Err(_) => {
            validate_err!("latitude must be a decimal value.")
        }
        Ok(distance) => {
            if distance < -90.0 || distance > 90.0 {
                validate_err!("latitude must be between -90 and 90 degrees.")
            } else {
                Ok(latitude)
            }
        }
    }
}

/// Validate a location longitude.
///
/// # Arguments
///
/// * `longitude` is the location longitude.
///
pub fn longitude(longitude: &str) -> crate::Result<String> {
    let longitude = validate_not_empty!("longitude", longitude);
    match longitude.parse::<f64>() {
        Err(_) => {
            validate_err!("longitude must be a decimal value.")
        }
        Ok(distance) => {
            if distance < -180.0 || distance > 180.0 {
                validate_err!("longitude must be between -180 and 180 degrees.")
            } else {
                Ok(longitude)
            }
        }
    }
}

/// Validate a locations timezone.
///
/// # Arguments
///
/// * `tz_name` is the location timezone name.
///
pub fn tz(tz_name: &str) -> crate::Result<String> {
    let tz_name = validate_not_empty!("tz", tz_name).to_lowercase();
    match chrono_tz::TZ_VARIANTS.iter().position(|tz| tz_name == tz.name().to_lowercase()) {
        Some(position) => Ok(chrono_tz::TZ_VARIANTS[position].name().to_string()),
        None => {
            validate_err!("timezone name '{}' is not valid.", tz_name)
        }
    }
}

#[cfg(test)]
mod tests {

    macro_rules! validate_not_empty_testcases {
        ($testcase: ident, $error_pattern: literal) => {
            assert!($testcase("").is_err());
            if let Err(error) = $testcase("") {
                assert!(error.to_string().contains($error_pattern));
            }
            assert!($testcase(" ").is_err());
        };
    }

    #[test]
    fn country_name() {
        let testcase = super::country_name;
        validate_not_empty_testcases!(testcase, "country name");
        assert_eq!(testcase(" Country Name ").unwrap(), "Country Name");
    }

    #[test]
    fn country_code() {
        let testcase = super::country_code;
        validate_not_empty_testcases!(testcase, "country code");
        assert_eq!(testcase(" Country Code ").unwrap(), "Country Code");
    }

    #[test]
    fn region_name() {
        let testcase = super::region_name;
        validate_not_empty_testcases!(testcase, "region name");
        assert_eq!(testcase(" Region Name ").unwrap(), "Region Name");
    }

    #[test]
    fn region_code() {
        let testcase = super::region_code;
        validate_not_empty_testcases!(testcase, "region code");
        assert_eq!(testcase(" Region Code ").unwrap(), "Region Code");
    }

    #[test]
    fn city_name() {
        let testcase = super::city_name;
        validate_not_empty_testcases!(testcase, "city name");
        assert_eq!(testcase(" Name City ").unwrap(), "Name City");
    }

    #[test]
    fn alias() {
        let testcase = super::alias;
        validate_not_empty_testcases!(testcase, "alias");
        assert_eq!(testcase(" Alias ").unwrap(), "alias");
    }

    #[test]
    fn latitude() {
        let testcase = super::latitude;
        validate_not_empty_testcases!(testcase, "latitude");
        assert_eq!(testcase(" 90 ").unwrap(), "90");
        assert!(testcase("90.0000000001").is_err());
        assert_eq!(testcase("-90").unwrap(), "-90");
        assert!(testcase("-90.0000000001").is_err());
        assert!(testcase("abc").is_err());
    }

    #[test]
    fn longitude() {
        let testcase = super::longitude;
        validate_not_empty_testcases!(testcase, "longitude");
        assert_eq!(testcase(" 180 ").unwrap(), "180");
        assert!(testcase("180.0000000001 ").is_err());
        assert_eq!(testcase("-180 ").unwrap(), "-180");
        assert!(testcase("-180.0000000001 ").is_err());
        assert!(testcase("abc").is_err());
    }

    #[test]
    fn tz() {
        let testcase = super::tz;
        validate_not_empty_testcases!(testcase, "tz");
        assert_eq!(testcase(" utc ").unwrap(), "UTC");
        assert!(testcase("some TZ").is_err());
    }
}
