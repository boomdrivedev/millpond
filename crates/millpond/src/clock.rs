//! The clock: now, from the system, or from `MILLPOND_NOW` where it names
//! an RFC 3339 time, so a test or a person replaying a scenario fixes it.

use jiff::tz::TimeZone;
use jiff::{Timestamp, Zoned};
use time::OffsetDateTime;

/// The variable that fixes now.
pub const NOW: &str = "MILLPOND_NOW";

/// Now, as an instant.
pub fn now() -> Timestamp {
    std::env::var(NOW)
        .ok()
        .and_then(|text| text.parse().ok())
        .unwrap_or_else(Timestamp::now)
}

/// Now in the machine's time zone, the zone a schedule is read in (C.4).
pub fn zoned() -> Zoned {
    now().to_zoned(zone())
}

/// The machine's time zone.
pub fn zone() -> TimeZone {
    TimeZone::system()
}

/// Now, as `time` gives it to weir's take and write protocol.
pub fn offset() -> OffsetDateTime {
    let now = now();
    OffsetDateTime::from_unix_timestamp_nanos(now.as_nanosecond())
        .unwrap_or_else(|_| OffsetDateTime::now_utc())
}

/// Now in RFC 3339, in UTC, to the second.
pub fn stamp() -> String {
    now().strftime("%Y-%m-%dT%H:%M:%SZ").to_string()
}
