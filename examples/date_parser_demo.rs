// Example demonstrating the natural language date parser
//
// Note: The parser returns UTC timestamps (Timestamp).
// For display purposes, we convert back to the system timezone (Zoned).
// This ensures "today" displays as the current date in local time,
// even though the underlying timestamp is midnight local time converted to UTC.
//
// Example: If system timezone is CET (UTC+1) on 2026-01-16:
//   - "today" returns 2026-01-15T23:00:00Z (midnight CET in UTC)
//   - Converted to CET: 2026-01-16T00:00:00+01:00 (correct civil date)

use zet::core::date_parser::NaturalDateParser;
use jiff::{Timestamp, tz::TimeZone};

fn main() {
    let now = Timestamp::now();
    let tz = TimeZone::system();

    println!("Current time: {}\n", now);

    let examples = vec![
        "today",
        "tomorrow",
        "yesterday",
        "in 3 days",
        "in 2 weeks",
        "3 days ago",
        "next monday",
        "last friday",
        "this friday",
        "tomorrow at 10:30 am",
        "next friday at 10:13 pm",
        "in 2 hours",
        "30 minutes ago",
    ];

    for example in examples {
        match NaturalDateParser::parse(example, now) {
            Ok(timestamp) => {
                // Convert back to system timezone for display
                let zoned = timestamp.to_zoned(tz.clone());
                println!("{:25} -> {} (UTC: {})", example, zoned, timestamp);
            }
            Err(e) => {
                println!("{:25} -> ERROR: {:?}", example, e);
            }
        }
    }
}
