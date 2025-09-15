//! Capture the `chrono` date and time usages to this module.
//!
//! The library changes between 0.4.22 and 0.4.24 introduced deprecation
//! of functions that were being used. So to isolate the library better,
//! utilities for the package was moved here.
use super::{Error, Result};
use chrono::prelude::*;
use chrono_tz::Tz;

/// Creates an ISO8601 date string.
///
/// The returned string will be formatted as YYYY-MM-DD where:
/// * YYYY is the 4 digit year
/// * MM is the month
/// * DD is the day in the month.
///
/// # Arguments
///
/// * `$date` the UTC date that will be converted.
///
pub fn isodate(date: &NaiveDate) -> String {
    fmt_date(date, "%Y-%m-%d")
}

/// Creates a date string using the provided date format.
///
/// This function is exposed to consolidate the `chrono` crate dependency to this crate.
/// The format must be a valid `chrono` date formatting string.
///
/// # Arguments
///
/// * `date` the UTC date timestamp to use.
/// * `format` the date format description.
///
pub fn fmt_date(date: &NaiveDate, format: &str) -> String {
    date.format(format).to_string()
}

/// Converts a date string to a UTC date.
///
/// The date can have the following forms:
///
/// * `YYYY-MM-DD` - where YYYY is the 4 digit year, MM is the 2 digit month, and DD the 2 digit
/// day of month.
/// * `MM-DD-YYYY` - where MM is the 2 digit month, DD is the 2 digit day of month, and YYYY is the
/// 4 digit year.
/// * `MMM-DD-YYYY` - where MMM is the abbreviated month name (always 3 characters), DD is the 2
/// digit day of month, and YYYY is the 4 digit year.
///
/// # Arguments
///
/// * `date_str` - the date string that will be validated.
///
/// An error will be returned if the date parsing fails.
pub fn parse_date(date_str: &str) -> Result<NaiveDate> {
    for fmt in ["%Y-%m-%d", "%m-%d-%Y", "%b-%d-%Y", "%m/%d/%Y"] {
        if let Ok(naive_date) = NaiveDate::parse_from_str(date_str, fmt) {
            return Ok(naive_date);
        }
    }
    let patterns = "YYYY-MM-DD, MM-DD-YYYY, MM/DD/YYYY, or MMM-DD-YYYY";
    Err(Error::from(format!("'{}' pattern must be {}.", date_str, patterns)))
}

/// A helper function that gets a timezone for a name of the timezone.
///
/// # Arguments
///
/// * `tz_name` is the timezone name.
pub fn get_tz(tz_name: &str) -> Result<Tz> {
    match tz_name.parse() {
        Ok(tz) => Ok(tz),
        Err(error) => Err(Error::from(error.to_string())),
    }
}

/// A boiler plate helper that creates a `NaiveDate` from a year, month, and day.
///
/// This really exists just to hide some of the changes that has happened in the 0.8.24
/// release. I really don't think there will be a need to worry about not just unwrapping
/// the result in my use case but this will atleast allow tracking a problem and not panicing.
/// If there is an error the default `NaiveDate` will be returned.
///
/// # Arguments
///
/// * `y` is the year of the date.
/// * `m` is the month of the year.
/// * `d` is the day of the month.
pub fn get_date(y: i32, m: u32, d: u32) -> NaiveDate {
    if let Some(nd) = NaiveDate::from_ymd_opt(y, m, d) {
        nd
    } else {
        // not the best solution but for this use case it's fine
        log::error!("Yikes... Bad date year={}, month={}, day={}, returning default!", y, m, d);
        NaiveDate::default()
    }
}

/// A boiler plate helper that creates a `NaiveTime` from hours, minutes, and seconds.
///
/// This really exists just to hide some of the changes that has happened in the 0.8.24
/// release. I really don't think there will be a need to worry about not just unwrapping
/// the result in my use case but this will atleast allow tracking a problem and not panicing.
/// If there is an error the default `NaiveTime` will be returned.
///
/// # Arguments
///
/// * `h` is the hour of the time.
/// * `m` is the minutes of the hour.
/// * `s` is the seconds of the minute.
pub fn get_time(h: u32, m: u32, s: u32) -> NaiveTime {
    if let Some(nt) = NaiveTime::from_hms_opt(h, m, s) {
        nt
    } else {
        // not the best solution but for this use case it's fine
        log::error!("Yikes... Bad time hour={}, minute={}, second={}, returning default!", h, m, s);
        NaiveTime::default()
    }
}

/// A boiler plate helper that creates a local `DateTime` from a `NaiveDateTime`.
///
/// This provides common functionality to create a local `DateTime` that reflects
/// the provided date time. The timestamp will reflect the timezone offset from
/// UTC. If there is an error creating the local date time the `epoch` will be
/// returned.
///
/// # Arguments
///
/// * `local` is the date and time of the local date time.
pub fn get_local_datetime(local: &NaiveDateTime) -> DateTime<Local> {
    match Local.from_local_datetime(local) {
        chrono::LocalResult::Single(dt) => dt,
        _ => {
            log::error!("Yikes... {} could not be converted to tz, forcing epoch", local);
            get_local_ts(0)
        }
    }
}

/// A boiler plate helper that creates a local `DateTime` from a timestamp.
///
/// This provides common functionality to create a local `DateTime` that reflects
/// the number of seconds from the `epoch`. If there is an error creating the local
/// date time the `epoch` will be returned.
///
/// # Arguments
///
/// * `ts` is the number of seconds from the `epoch`.
pub fn get_local_ts(ts: i64) -> DateTime<Local> {
    match Local.timestamp_opt(ts, 0) {
        chrono::LocalResult::Single(dt) => dt,
        _ => {
            log::error!("Yikes... {} could not be converted to tz, forcing epoch", ts);
            Local.timestamp_opt(0, 0).unwrap()
        }
    }
}

/// A boiler plate helper that creates a `DateTime` from a `NaiveDateTime` for a
/// timezone.
///
/// This provides common functionality to create a `DateTime` for a timezone that reflects the
/// provided date time. The timestamp will reflect the timezone offset from UTC. If there is an
/// error creating the date time, for a timezone, the returned date time will reflect the offset
/// from the `epoch`.
///
/// # Arguments
///
/// * `local` is the date and time of the local date time.
/// * `tz` is the timezone that will be used to create the date time.
pub fn get_tz_datetime(local: &NaiveDateTime, tz: &Tz) -> DateTime<Tz> {
    match tz.from_local_datetime(local) {
        chrono::LocalResult::Single(dt) => dt,
        _ => {
            log::error!("Yikes... {} could not be converted to tz, forcing epoch", local);
            get_tz_ts(0, tz)
        }
    }
}

/// A boiler plate helper that creates a `DateTime` for a timezone from a timestamp.
///
/// This provides common functionality to create a local `DateTime` that reflects
/// the number of seconds from the `epoch`. If there is an error creating the date time,
/// for a timezone, the `epoch` date time will be returned for the timezone.
///
/// # Arguments
///
/// * `ts` is the number of seconds from the `epoch`.
/// * `tz` is the timezone that will be used to create the date time.
pub fn get_tz_ts(ts: i64, tz: &Tz) -> DateTime<Tz> {
    match tz.timestamp_opt(ts, 0) {
        chrono::LocalResult::Single(dt) => dt,
        _ => {
            log::error!("Yikes... {} could not be converted to tz, forcing epoch", ts);
            tz.timestamp_opt(0, 0).unwrap()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn date() {
        let date = NaiveDate::from_ymd_opt(2022, 10, 5).unwrap();
        assert_eq!(isodate(&date), "2022-10-05")
    }
    #[test]
    fn parse_dates() {
        assert_eq!(parse_date("2022-7-15").unwrap(), get_date(2022, 7, 15));
        assert_eq!(parse_date("7-1-2022").unwrap(), get_date(2022, 7, 1));
        assert_eq!(parse_date("7-1-22").unwrap(), get_date(7, 1, 22));
        assert_eq!(parse_date("1-7-22").unwrap(), get_date(1, 7, 22));
        assert_eq!(parse_date("jul-15-2022").unwrap(), get_date(2022, 7, 15));
        assert_eq!(parse_date("Jul-15-2022").unwrap(), get_date(2022, 7, 15));
        assert_eq!(parse_date("JUL-15-2022").unwrap(), get_date(2022, 7, 15));
        assert_eq!(parse_date("JUL-15-22").unwrap(), get_date(22, 7, 15));
        assert!(parse_date("JULY-15-22").is_err());
    }
    #[test]
    fn wtf() {
        // Here's some chrono samples that can verify the implementation.

        // Naive* defaults reflect the Unux epoch
        assert_eq!(NaiveDateTime::default().and_utc().timestamp(), 0);
        assert_eq!(NaiveDateTime::default().to_string(), "1970-01-01 00:00:00");
        assert_eq!(NaiveDate::default(), NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
        assert_eq!(NaiveTime::default(), NaiveTime::from_hms_opt(0, 0, 0).unwrap());

        // The NaiveDateTime is really just a Utc without an offset
        let utc = Utc.timestamp_opt(0, 0).unwrap();
        assert_eq!(utc.timestamp(), 0);
        assert_eq!(utc.to_string(), "1970-01-01 00:00:00 UTC");

        // the NaiveDateTime is used to create DateTime instances
        let ndt = NaiveDateTime::new(get_date(2023, 5, 5), get_time(12, 0, 0));
        // HEY!!! this will panic >>> eprintln!("{}", ndt.format(fmt));
        let fmt = "%Y-%m-%d %H:%M:%S %z";

        // using these traits give NaiveDateTime and DateTime access to the components
        use chrono::{Datelike, Timelike};
        assert_eq!(ndt.year(), 2023);
        assert_eq!(ndt.month(), 5);
        assert_eq!(ndt.day(), 5);
        assert_eq!(ndt.hour(), 12);
        assert_eq!(ndt.minute(), 0);
        assert_eq!(ndt.second(), 0);

        // Utc uses a FixedOffset of +0000
        let utc = Utc.from_utc_datetime(&ndt);
        assert_eq!(utc.format(fmt).to_string(), "2023-05-05 12:00:00 +0000");

        // Local uses a FixedOffset corresponding to the TimeZone at that particular date and time
        let nyd = NaiveDateTime::new(get_date(2023, 1, 1), NaiveTime::default());
        assert!(get_local_datetime(&nyd).format(fmt).to_string().starts_with("2023-01-01 00:00:00"));
        let localtime = Local.from_utc_datetime(&ndt);
        assert_eq!(localtime.format(fmt).to_string(), "2023-05-05 05:00:00 -0700");

        // Tz uses a TzOffset which has both a Utc offset and a daylight savings offset.
        let mt_tz = get_tz("America/Denver").unwrap();
        assert_eq!(get_tz_datetime(&nyd, &mt_tz).format(fmt).to_string(), "2023-01-01 00:00:00 -0700");
        let mt = mt_tz.from_utc_datetime(&ndt);
        assert_eq!(mt.format(fmt).to_string(), "2023-05-05 06:00:00 -0600");

        let pt_tz = get_tz("America/Los_Angeles").unwrap();
        assert_eq!(get_tz_datetime(&nyd, &pt_tz).format(fmt).to_string(), "2023-01-01 00:00:00 -0800");
        let pt = pt_tz.from_utc_datetime(&ndt);
        assert_eq!(pt.format(fmt).to_string(), "2023-05-05 05:00:00 -0700");

        // timestamps all reflect the epoch seconds
        assert_eq!(ndt.and_utc().timestamp(), utc.timestamp());
        assert_eq!(utc.timestamp(), localtime.timestamp());
        assert_eq!(utc.timestamp(), mt.timestamp());
        assert_eq!(utc.timestamp(), pt.timestamp());

        // timestamps can be used to create a date time for a particular timezone
        let ts = ndt.and_utc().timestamp();
        assert_eq!(get_local_ts(ts), localtime);
        assert_eq!(get_tz_ts(ts, &mt_tz), mt);
        assert_eq!(get_tz_ts(ts, &pt_tz), pt);
    }
}
