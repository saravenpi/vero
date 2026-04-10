use chrono::{DateTime, Utc};

pub(crate) fn parse_email_timestamp(date: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc2822(date)
        .ok()
        .map(|timestamp| timestamp.with_timezone(&Utc))
        .or_else(|| {
            DateTime::parse_from_rfc3339(date)
                .ok()
                .map(|timestamp| timestamp.with_timezone(&Utc))
        })
}
