/// Creates an error when validation fails.
macro_rules! validate_err {
    ($($arg:tt)*) => {
        Err(crate::Error::from(format!($($arg)*)))
    };
}

/// Validate a locations city name.
///
/// # Arguments
///
/// * `name` is the city name.
///
pub fn city(name: &str) -> crate::Result<String> {
    let city = name.trim().to_string();
    if city.is_empty() {
        validate_err!("location city name cannot be empty")
    } else {
        Ok(city)
    }
}

/// Validate a locations abbreviated state name.
///
/// # Arguments
///
/// * `name` is the location abbreviated state name.
///
pub fn state_id(name: &str) -> crate::Result<String> {
    let city = name.trim().to_string();
    if city.is_empty() {
        validate_err!("location abbreviated state name cannot be empty")
    } else {
        Ok(city)
    }
}

/// Validate a locations state name.
///
/// # Arguments
///
/// * `name` is the location state name.
///
pub fn state(name: &str) -> crate::Result<String> {
    let city = name.trim().to_string();
    if city.is_empty() {
        validate_err!("location state name cannot be empty")
    } else {
        Ok(city)
    }
}

/// Validate a locations name.
///
/// # Arguments
///
/// * `name` is the location name.
///
pub fn name(name: &str) -> crate::Result<String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        validate_err!("location name cannot be empty")
    } else {
        Ok(name)
    }
}

/// Validate a locations alias name.
///
/// # Arguments
///
/// * `alias` is the locations alias name.
///
pub fn alias(alias: &str) -> crate::Result<String> {
    let alias = alias.trim().to_lowercase();
    if alias.is_empty() {
        validate_err!("alias name cannot be empty.")
    } else {
        Ok(alias)
    }
}

/// Validate a locations latitude.
///
/// # Arguments
///
/// * `latitude` is the location latitude.
///
pub fn latitude(latitude: &str) -> crate::Result<String> {
    let latitude = latitude.trim().to_string();
    if latitude.is_empty() {
        validate_err!("latitude cannot be empty.")
    } else {
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
}

/// Validate a locations longitude.
///
/// # Arguments
///
/// * `longitude` is the location longitude.
///
pub fn longitude(longitude: &str) -> crate::Result<String> {
    let longitude = longitude.trim().to_string();
    if longitude.is_empty() {
        validate_err!("longitude cannot be empty.")
    } else {
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
}

/// Validate a locations timezone.
///
/// # Arguments
///
/// * `tz_name` is the location timezone name.
///
pub fn tz(tz_name: &str) -> crate::Result<String> {
    let tz_name = tz_name.trim().to_lowercase();
    if tz_name.is_empty() {
        validate_err!("timezone cannot be empty.")
    } else {
        match chrono_tz::TZ_VARIANTS.iter().position(|tz| tz_name == tz.name().to_lowercase()) {
            Some(position) => Ok(chrono_tz::TZ_VARIANTS[position].name().to_string()),
            None => {
                validate_err!("timezone name '{}' is not valid.", tz_name)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_validator() {
        assert_eq!(name(" test ").unwrap(), "test");
        assert!(name("").is_err());
        assert!(name(" ").is_err());
    }

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
