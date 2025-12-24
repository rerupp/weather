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
            validate_err!("location {} cannot be empty.", $what)?;
        }
        value.to_string()
    }};
}

/// Validate a locations city name.
///
/// # Arguments
///
/// * `city_name` is the name of the city.
///
pub fn city(city_name: &str) -> crate::Result<String> {
    let city_name = validate_not_empty!("city name", city_name);
    Ok(city_name)
}

/// Validate a locations abbreviated state name.
///
/// # Arguments
///
/// * `state_id` is the location abbreviated state name.
///
pub fn state_id(state_id: &str) -> crate::Result<String> {
    let state_id = validate_not_empty!("state ID", state_id);
    Ok(state_id)
}

/// Validate a locations state name.
///
/// # Arguments
///
/// * `state_name` is the location state name.
///
pub fn state(state_name: &str) -> crate::Result<String> {
    let state_name = validate_not_empty!("state name", state_name);
    Ok(state_name)
}

/// Validate a locations alias name.
///
/// # Arguments
///
/// * `alias` is the locations alias name.
///
pub fn alias(alias: &str) -> crate::Result<String> {
    let alias = validate_not_empty!("alias", alias).to_lowercase();
    Ok(validate_not_empty!("alias", alias))
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
    use super::*;

    #[test]
    fn alias_validator() {
        assert_eq!(alias(" TEST ").unwrap(), "test");
        assert!(alias("").is_err());
        assert!(alias(" ").is_err());
    }

    #[test]
    fn latitude_validator() {
        assert_eq!(latitude(" 90 ").unwrap(), "90");
        assert!(latitude("90.0000000001").is_err());
        assert_eq!(latitude("-90").unwrap(), "-90");
        assert!(latitude("-90.0000000001").is_err());
        assert!(latitude("").is_err());
        assert!(latitude(" ").is_err());
        assert!(latitude("abc").is_err());
    }

    #[test]
    fn longitude_validator() {
        assert_eq!(longitude(" 180 ").unwrap(), "180");
        assert!(longitude("180.0000000001 ").is_err());
        assert_eq!(longitude("-180 ").unwrap(), "-180");
        assert!(longitude("-180.0000000001 ").is_err());
        assert!(longitude("").is_err());
        assert!(longitude(" ").is_err());
        assert!(longitude("abc").is_err());
    }

    #[test]
    fn tz_validator() {
        assert_eq!(tz(" utc ").unwrap(), "UTC");
        assert!(tz("").is_err());
        assert!(tz(" ").is_err());
        assert!(tz("some TZ").is_err());
    }
}
