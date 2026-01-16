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
        // Basic patterns
        "today",
        "tomorrow",
        "yesterday",

        // "in X" pattern - all time strides
        "in 30 seconds",
        "in 15 minutes",
        "in 2 hours",
        "in 3 days",
        "in 2 weeks",
        "in 6 months",
        "in 2 years",

        // "X from now" pattern
        "5 days from now",
        "2 weeks from now",

        // "X ago" pattern - all time strides
        "10 seconds ago",
        "30 minutes ago",
        "3 hours ago",
        "3 days ago",
        "2 weeks ago",
        "6 months ago",
        "1 years ago",

        // Weekday patterns
        "next monday",
        "this friday",
        "last friday",
        "on wednesday",

        // Month patterns
        "next march",
        "last december",

        // Week/year patterns
        "next week",
        "this week",
        "last week",
        "next year",
        "this year",
        "last year",

        // With time specifications
        "today at 2 pm",
        "tomorrow at 10:30 am",
        "next friday at 10:13 pm",
        "in 3 days at 8:30 am",
        "on monday at 7:15 am",
        "tomorrow at 12 am",  // midnight
        "tomorrow at 12 pm",  // noon

        // Number words
        "in three days",
        "in five weeks",
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
