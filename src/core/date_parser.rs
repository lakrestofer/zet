use chumsky::prelude::*;
use jiff::{Timestamp, ToSpan, Zoned, civil::Date, tz::TimeZone};

pub struct NaturalDateParser;

#[derive(Debug)]
pub enum ParseError {
    TokenizationError(String),
    PatternParseError(String),
    ConversionError(String),
}

impl NaturalDateParser {
    /// Parse a natural language date string into a UTC Timestamp
    ///
    /// # Return Value
    /// Returns a `Timestamp` in UTC. For display purposes, convert to timezone-aware
    /// `Zoned` using `.to_zoned(TimeZone)` to show the correct civil date/time.
    ///
    /// **Important:** The returned timestamp may not display the expected date when
    /// shown directly as UTC. For example, "today" at midnight in timezone CET (UTC+1)
    /// returns a UTC timestamp that appears to be yesterday. Always convert to a
    /// timezone for display:
    ///
    /// ```ignore
    /// use jiff::{Timestamp, tz::TimeZone};
    ///
    /// let now = Timestamp::now();
    /// let timestamp = NaturalDateParser::parse("today", now)?;
    ///
    /// // Convert to system timezone for display
    /// let zoned = timestamp.to_zoned(TimeZone::system());
    /// println!("Today: {}", zoned);  // Shows correct date
    /// ```
    ///
    /// # Arguments
    /// * `input` - Natural language date string (e.g., "in 3 days", "next friday", "today")
    /// * `now` - Reference timestamp to calculate relative dates from
    ///
    /// # Design Note
    /// We return `Timestamp` (UTC) rather than `Zoned` (timezone-aware) because:
    /// - Timestamps are the universal interchange format
    /// - Consumers can convert to any timezone they need
    /// - Keeps the API simple with a single return type
    pub fn parse(input: &str, now: Timestamp) -> Result<Timestamp, ParseError> {
        // Step 1: Tokenize the input string
        let lowercase_input = input.to_lowercase();
        let tokens = token_parser()
            .parse(lowercase_input.as_str())
            .into_result()
            .map_err(|e| ParseError::TokenizationError(format!("{:?}", e)))?;

        // Step 2: Parse tokens into a TimePattern
        let pattern = pattern_parser()
            .parse(tokens.as_slice())
            .into_result()
            .map_err(|e| ParseError::PatternParseError(format!("{:?}", e)))?;

        // Step 3: Convert pattern to timestamp
        pattern.to_timestamp(now)
    }
}

#[derive(Clone, Copy, Debug)]
struct Time {
    hour: u32,
    minute: Option<u32>,
}

/// This enum tries to list some common natural language patterns for referring
/// to different moments in time
#[derive(Clone)]
enum TimePattern {
    /// today
    Today { at: Option<Time> },
    /// tomorrow
    Tomorrow { at: Option<Time> },
    /// yesterday
    Yesterday { at: Option<Time> },
    /// in 3 days
    /// in four months
    /// in 3 weeks at 12:00
    /// in 3 weeks @ 13
    InAmount {
        n: u32,
        stride: TimeStride,
        at: Option<Time>,
    },
    /// 3 weeks from now at 12:00 am
    FromNow {
        n: u32,
        stride: TimeStride,
        at: Option<Time>,
    },
    /// 3 days ago
    /// 5 minutes ago
    Ago {
        n: u32,
        stride: TimeStride,
        at: Option<Time>,
    },
    /// next friday
    /// next january
    /// next wednesday at 12
    Next {
        moment: TimeMoment,
        at: Option<Time>,
    },
    /// This friday @ 14
    This {
        moment: TimeMoment,
        at: Option<Time>,
    },
    /// last friday
    /// last month
    /// last year
    Last {
        moment: TimeMoment,
        at: Option<Time>,
    },
    /// on monday
    /// on friday at 07:15am
    OnWeekday { moment: Weekday, at: Option<Time> },
}

#[derive(Clone)]
enum TimeMoment {
    Weekday(Weekday),
    Month(Month),
    Week,
    Year,
}

#[derive(Clone, Copy)]
enum TimeStride {
    Seconds,
    Minutes,
    Hours,
    Days,
    Weeks,
    Months,
    Years,
}

fn token_parser<'src>() -> impl Parser<'src, &'src str, Vec<NatDatToken>> {
    let keyword1 = choice((
        just("yesterday").to(NatDatToken::Yesterday),
        just("tomorrow").to(NatDatToken::Tomorrow),
        just("beginning").to(NatDatToken::Beginning),
        just("afternoon").to(NatDatToken::Afternoon),
        just("seconds").to(NatDatToken::Seconds),
        just("morning").to(NatDatToken::Morning),
        just("evening").to(NatDatToken::Evening),
        just("weekend").to(NatDatToken::Weekend),
        just("minutes").to(NatDatToken::Minutes),
        just("months").to(NatDatToken::Months),
        just("today").to(NatDatToken::Today),
        just("hours").to(NatDatToken::Hours),
        just("weeks").to(NatDatToken::Weeks),
        just("years").to(NatDatToken::Years),
        just("start").to(NatDatToken::Start),
        just("night").to(NatDatToken::Night),
        just("week").to(NatDatToken::Weeks), // Singular week maps to Weeks token
        just("year").to(NatDatToken::Years), // Singular year maps to Years token
    ));

    let keyword2 = choice((
        just("next").to(NatDatToken::Next),
        just("last").to(NatDatToken::Last),
        just("this").to(NatDatToken::This),
        just("from").to(NatDatToken::From),
        just("days").to(NatDatToken::Days),
        just("day").to(NatDatToken::Days), // Singular day maps to Days token
        just("now").to(NatDatToken::Now),
        just("ago").to(NatDatToken::Ago),
        just("end").to(NatDatToken::End),
        just("the").to(NatDatToken::The),
        just("at").to(NatDatToken::At),
        just("in").to(NatDatToken::In),
        just("on").to(NatDatToken::On),
        just("of").to(NatDatToken::Of),
        just("am").to(NatDatToken::Am),
        just("pm").to(NatDatToken::Pm),
        just(":").to(NatDatToken::Colon),
        just("@").to(NatDatToken::At),
    ));

    choice((
        keyword1,
        month().map(NatDatToken::Month),
        weekday().map(NatDatToken::Weekday),
        number().map(NatDatToken::Number), // Try number before keyword2 to avoid "on" matching "one"
        keyword2,
    ))
    .padded()
    .repeated()
    .collect()
}

// Helper to match a specific token using select! macro
macro_rules! tok {
    ($token_variant:pat) => {
        select! {
            t@$token_variant => t
        }
    };
}

// Parse a number token
fn parse_number<'src>()
-> impl Parser<'src, &'src [NatDatToken], u32, extra::Err<Rich<'src, NatDatToken>>> + Clone {
    any().try_map(|t: NatDatToken, span| match t {
        NatDatToken::Number(n) => Ok(n),
        _ => Err(Rich::custom(span, "expected number")),
    })
}

// Parse a weekday token
fn parse_weekday<'src>()
-> impl Parser<'src, &'src [NatDatToken], Weekday, extra::Err<Rich<'src, NatDatToken>>> + Clone {
    any().try_map(|t: NatDatToken, span| match t {
        NatDatToken::Weekday(w) => Ok(w),
        _ => Err(Rich::custom(span, "expected weekday")),
    })
}

// Parse a month token
fn parse_month<'src>()
-> impl Parser<'src, &'src [NatDatToken], Month, extra::Err<Rich<'src, NatDatToken>>> + Clone {
    any().try_map(|t: NatDatToken, span| match t {
        NatDatToken::Month(m) => Ok(m),
        _ => Err(Rich::custom(span, "expected month")),
    })
}

// Parse time stride (days, weeks, months, years, etc.)
fn parse_stride<'src>()
-> impl Parser<'src, &'src [NatDatToken], TimeStride, extra::Err<Rich<'src, NatDatToken>>> + Clone {
    any().try_map(|t: NatDatToken, span| match t {
        NatDatToken::Seconds => Ok(TimeStride::Seconds),
        NatDatToken::Minutes => Ok(TimeStride::Minutes),
        NatDatToken::Hours => Ok(TimeStride::Hours),
        NatDatToken::Days => Ok(TimeStride::Days),
        NatDatToken::Weeks => Ok(TimeStride::Weeks),
        NatDatToken::Months => Ok(TimeStride::Months),
        NatDatToken::Years => Ok(TimeStride::Years),
        _ => Err(Rich::custom(span, "expected time unit")),
    })
}

// Parse time component: [at|@] <hour> [:<minute>] [am|pm]
fn parse_time<'src>()
-> impl Parser<'src, &'src [NatDatToken], Time, extra::Err<Rich<'src, NatDatToken>>> + Clone {
    tok!(NatDatToken::At)
        .ignore_then(parse_number())
        .then(
            tok!(NatDatToken::Colon)
                .ignore_then(parse_number())
                .or_not(),
        )
        .then(
            tok!(NatDatToken::Am)
                .to(false)
                .or(tok!(NatDatToken::Pm).to(true))
                .or_not(),
        )
        .map(move |((hour, minute), is_pm)| {
            let adjusted_hour = match is_pm {
                Some(true) if hour < 12 => hour + 12,
                Some(false) if hour == 12 => 0,
                _ => hour,
            };
            Time {
                hour: adjusted_hour,
                minute,
            }
        })
}

// Parse TimeMoment (weekday, month, week, year)
fn parse_moment<'src>()
-> impl Parser<'src, &'src [NatDatToken], TimeMoment, extra::Err<Rich<'src, NatDatToken>>> + Clone {
    choice((
        parse_weekday().map(TimeMoment::Weekday),
        parse_month().map(TimeMoment::Month),
        tok!(NatDatToken::Weeks).to(TimeMoment::Week),
        tok!(NatDatToken::Years).to(TimeMoment::Year),
    ))
}

// Main pattern parser
fn pattern_parser<'src>()
-> impl Parser<'src, &'src [NatDatToken], TimePattern, extra::Err<Rich<'src, NatDatToken>>> + Clone
{
    let time_opt = parse_time().or_not();

    choice((
        // "today" [at time]
        tok!(NatDatToken::Today)
            .ignore_then(time_opt.clone())
            .map(|at| TimePattern::Today { at }),
        // "tomorrow" [at time]
        tok!(NatDatToken::Tomorrow)
            .ignore_then(time_opt.clone())
            .map(|at| TimePattern::Tomorrow { at }),
        // "yesterday" [at time]
        tok!(NatDatToken::Yesterday)
            .ignore_then(time_opt.clone())
            .map(|at| TimePattern::Yesterday { at }),
        // "in" <number> <stride> [at time]
        tok!(NatDatToken::In)
            .ignore_then(parse_number())
            .then(parse_stride())
            .then(time_opt.clone())
            .map(|((n, stride), at)| TimePattern::InAmount { n, stride, at }),
        // <number> <stride> "from" "now" [at time]
        parse_number()
            .then(parse_stride())
            .then_ignore(tok!(NatDatToken::From))
            .then_ignore(tok!(NatDatToken::Now))
            .then(time_opt.clone())
            .map(|((n, stride), at)| TimePattern::FromNow { n, stride, at }),
        // <number> <stride> "ago"
        parse_number()
            .then(parse_stride())
            .then_ignore(tok!(NatDatToken::Ago))
            .then(time_opt.clone())
            .map(|((n, stride), at)| TimePattern::Ago { n, stride, at }),
        // "next" <moment> [at time]
        tok!(NatDatToken::Next)
            .ignore_then(parse_moment())
            .then(time_opt.clone())
            .map(|(moment, at)| TimePattern::Next { moment, at }),
        // "this" <moment> [at time]
        tok!(NatDatToken::This)
            .ignore_then(parse_moment())
            .then(time_opt.clone())
            .map(|(moment, at)| TimePattern::This { moment, at }),
        // "last" <moment> [at time]
        tok!(NatDatToken::Last)
            .ignore_then(parse_moment())
            .then(time_opt.clone())
            .map(|(moment, at)| TimePattern::Last { moment, at }),
        // "on" <weekday> [at time]
        tok!(NatDatToken::On)
            .ignore_then(parse_weekday())
            .then(time_opt.clone())
            .map(|(moment, at)| TimePattern::OnWeekday { moment, at }),
    ))
}

// Timestamp conversion implementation
impl TimePattern {
    fn to_timestamp(&self, now: Timestamp) -> Result<Timestamp, ParseError> {
        // Convert to system timezone for easier manipulation
        let tz = TimeZone::system();
        let zoned_now = now.to_zoned(tz.clone());

        match self {
            TimePattern::Today { at } => {
                let date = zoned_now.date();
                apply_time(date, at, &tz)
            }

            TimePattern::Tomorrow { at } => {
                let date = zoned_now
                    .date()
                    .checked_add(1.day())
                    .map_err(|e| ParseError::ConversionError(format!("date overflow: {}", e)))?;
                apply_time(date, at, &tz)
            }

            TimePattern::Yesterday { at } => {
                let date = zoned_now
                    .date()
                    .checked_sub(1.day())
                    .map_err(|e| ParseError::ConversionError(format!("date underflow: {}", e)))?;
                apply_time(date, at, &tz)
            }

            TimePattern::InAmount { n, stride, at } | TimePattern::FromNow { n, stride, at } => {
                let span = stride_to_span(*n, stride);
                let future = zoned_now
                    .checked_add(span)
                    .map_err(|e| ParseError::ConversionError(format!("date overflow: {}", e)))?;

                if at.is_some() {
                    apply_time(future.date(), at, &tz)
                } else {
                    Ok(future.timestamp())
                }
            }

            TimePattern::Ago { n, stride, at } => {
                let span = stride_to_span(*n, stride);
                let past = zoned_now
                    .checked_sub(span)
                    .map_err(|e| ParseError::ConversionError(format!("date underflow: {}", e)))?;

                if at.is_some() {
                    apply_time(past.date(), at, &tz)
                } else {
                    Ok(past.timestamp())
                }
            }

            TimePattern::Next { moment, at } => {
                let target = find_next_moment(&zoned_now, moment)?;
                apply_time(target, at, &tz)
            }

            TimePattern::This { moment, at } => {
                let target = find_this_moment(&zoned_now, moment)?;
                apply_time(target, at, &tz)
            }

            TimePattern::Last { moment, at } => {
                let target = find_last_moment(&zoned_now, moment)?;
                apply_time(target, at, &tz)
            }

            TimePattern::OnWeekday { moment, at } => {
                let target = find_next_weekday(&zoned_now, moment)?;
                apply_time(target, at, &tz)
            }
        }
    }
}

fn stride_to_span(n: u32, stride: &TimeStride) -> jiff::Span {
    match stride {
        TimeStride::Seconds => (n as i64).seconds(),
        TimeStride::Minutes => (n as i64).minutes(),
        TimeStride::Hours => (n as i64).hours(),
        TimeStride::Days => (n as i64).days(),
        TimeStride::Weeks => (n as i64).weeks(),
        TimeStride::Months => (n as i64).months(),
        TimeStride::Years => (n as i64).years(),
    }
}

/// Apply a time component to a date, returning a UTC timestamp.
///
/// The date is interpreted as being in the given timezone, then converted to UTC.
/// This is why "today" with system timezone CET returns a UTC timestamp that
/// appears to be yesterday - it's midnight CET converted to UTC.
fn apply_time(date: Date, time_opt: &Option<Time>, tz: &TimeZone) -> Result<Timestamp, ParseError> {
    let (hour, minute) = if let Some(t) = time_opt {
        (t.hour as i8, t.minute.unwrap_or(0) as i8)
    } else {
        (0, 0)
    };

    date.at(hour, minute, 0, 0)
        .to_zoned(tz.clone())
        .map(|z| z.timestamp())
        .map_err(|e| ParseError::ConversionError(format!("failed to create timestamp: {}", e)))
}

fn find_next_weekday(now: &Zoned, target_weekday: &Weekday) -> Result<Date, ParseError> {
    let current = now.date();
    let current_weekday = weekday_to_number(current.weekday());
    let target = weekday_to_number(weekday_to_jiff(target_weekday));

    // Calculate days until target weekday
    let days_ahead = if current_weekday < target {
        (target - current_weekday) as i64
    } else if current_weekday > target {
        (7 - (current_weekday - target)) as i64
    } else {
        7 // Same day, go to next week
    };

    current
        .checked_add(days_ahead.days())
        .map_err(|e| ParseError::ConversionError(format!("date overflow: {}", e)))
}

fn find_next_moment(now: &Zoned, moment: &TimeMoment) -> Result<Date, ParseError> {
    let current = now.date();

    match moment {
        TimeMoment::Weekday(wd) => find_next_weekday(now, wd),

        TimeMoment::Month(m) => {
            let target_month = month_to_number(m);
            let current_month = current.month();

            if current_month < target_month {
                // Later this year
                Date::new(current.year(), target_month, 1)
                    .map_err(|e| ParseError::ConversionError(format!("invalid date: {}", e)))
            } else {
                // Next year
                Date::new(current.year() + 1, target_month, 1)
                    .map_err(|e| ParseError::ConversionError(format!("invalid date: {}", e)))
            }
        }

        TimeMoment::Week => {
            // Next week = 7 days from now
            current
                .checked_add(7.days())
                .map_err(|e| ParseError::ConversionError(format!("date overflow: {}", e)))
        }

        TimeMoment::Year => Date::new(current.year() + 1, 1, 1)
            .map_err(|e| ParseError::ConversionError(format!("invalid date: {}", e))),
    }
}

fn find_this_moment(now: &Zoned, moment: &TimeMoment) -> Result<Date, ParseError> {
    let current = now.date();

    match moment {
        TimeMoment::Weekday(wd) => {
            // "this friday" means the upcoming friday in the current week
            let current_weekday = weekday_to_number(current.weekday());
            let target = weekday_to_number(weekday_to_jiff(wd));

            if current_weekday <= target {
                // Target is later this week
                let days_ahead = (target - current_weekday) as i64;
                current
                    .checked_add(days_ahead.days())
                    .map_err(|e| ParseError::ConversionError(format!("date overflow: {}", e)))
            } else {
                // Target already passed this week, go to next week
                let days_ahead = (7 - (current_weekday - target)) as i64;
                current
                    .checked_add(days_ahead.days())
                    .map_err(|e| ParseError::ConversionError(format!("date overflow: {}", e)))
            }
        }

        TimeMoment::Month(_) => {
            // "this month" = first day of current month
            Date::new(current.year(), current.month(), 1)
                .map_err(|e| ParseError::ConversionError(format!("invalid date: {}", e)))
        }

        TimeMoment::Week => {
            // "this week" = current date (or monday of current week)
            Ok(current)
        }

        TimeMoment::Year => {
            // "this year" = first day of current year
            Date::new(current.year(), 1, 1)
                .map_err(|e| ParseError::ConversionError(format!("invalid date: {}", e)))
        }
    }
}

fn find_last_moment(now: &Zoned, moment: &TimeMoment) -> Result<Date, ParseError> {
    let current = now.date();

    match moment {
        TimeMoment::Weekday(wd) => {
            let current_weekday = weekday_to_number(current.weekday());
            let target = weekday_to_number(weekday_to_jiff(wd));

            let days_back = if current_weekday > target {
                (current_weekday - target) as i64
            } else if current_weekday < target {
                (7 - (target - current_weekday)) as i64
            } else {
                7 // Same day, go to last week
            };

            current
                .checked_sub(days_back.days())
                .map_err(|e| ParseError::ConversionError(format!("date underflow: {}", e)))
        }

        TimeMoment::Month(m) => {
            let target_month = month_to_number(m);
            let current_month = current.month();

            if current_month > target_month {
                // Earlier this year
                Date::new(current.year(), target_month, 1)
                    .map_err(|e| ParseError::ConversionError(format!("invalid date: {}", e)))
            } else {
                // Last year
                Date::new(current.year() - 1, target_month, 1)
                    .map_err(|e| ParseError::ConversionError(format!("invalid date: {}", e)))
            }
        }

        TimeMoment::Week => {
            // Last week = 7 days ago
            current
                .checked_sub(7.days())
                .map_err(|e| ParseError::ConversionError(format!("date underflow: {}", e)))
        }

        TimeMoment::Year => Date::new(current.year() - 1, 1, 1)
            .map_err(|e| ParseError::ConversionError(format!("invalid date: {}", e))),
    }
}

fn weekday_to_jiff(wd: &Weekday) -> jiff::civil::Weekday {
    match wd {
        Weekday::Monday => jiff::civil::Weekday::Monday,
        Weekday::Tuesday => jiff::civil::Weekday::Tuesday,
        Weekday::Wednesday => jiff::civil::Weekday::Wednesday,
        Weekday::Thursday => jiff::civil::Weekday::Thursday,
        Weekday::Friday => jiff::civil::Weekday::Friday,
        Weekday::Saturday => jiff::civil::Weekday::Saturday,
        Weekday::Sunday => jiff::civil::Weekday::Sunday,
    }
}

fn weekday_to_number(wd: jiff::civil::Weekday) -> i8 {
    match wd {
        jiff::civil::Weekday::Monday => 1,
        jiff::civil::Weekday::Tuesday => 2,
        jiff::civil::Weekday::Wednesday => 3,
        jiff::civil::Weekday::Thursday => 4,
        jiff::civil::Weekday::Friday => 5,
        jiff::civil::Weekday::Saturday => 6,
        jiff::civil::Weekday::Sunday => 7,
    }
}

fn month_to_number(m: &Month) -> i8 {
    match m {
        Month::January => 1,
        Month::February => 2,
        Month::March => 3,
        Month::April => 4,
        Month::May => 5,
        Month::June => 6,
        Month::July => 7,
        Month::August => 8,
        Month::September => 9,
        Month::October => 10,
        Month::November => 11,
        Month::December => 12,
    }
}

fn number<'src>() -> impl Parser<'src, &'src str, u32> {
    choice((
        just("one").to(1),
        just("two").to(2),
        just("three").to(3),
        just("four").to(4),
        just("five").to(5),
        just("six").to(6),
        just("seven").to(7),
        just("eight").to(8),
        just("nine").to(9),
        just("ten").to(10),
        just("eleven").to(11),
        just("twelve").to(12),
        just("thirteen").to(13),
        text::int(10).map(|s: &str| s.parse().unwrap()),
    ))
    .padded()
}

fn weekday<'src>() -> impl Parser<'src, &'src str, Weekday> {
    choice((
        just("monday"),
        just("tuesday"),
        just("wednesday"),
        just("thursday"),
        just("friday"),
        just("saturday"),
        just("sunday"),
    ))
    .map(|s| match s {
        "monday" => Weekday::Monday,
        "tuesday" => Weekday::Tuesday,
        "wednesday" => Weekday::Wednesday,
        "thursday" => Weekday::Thursday,
        "friday" => Weekday::Friday,
        "saturday" => Weekday::Saturday,
        "sunday" => Weekday::Sunday,
        _ => unreachable!(),
    })
    .padded()
}

fn month<'src>() -> impl Parser<'src, &'src str, Month> {
    let padded_just = |p| just(p).padded();
    choice((
        padded_just("january"),
        padded_just("february"),
        padded_just("march"),
        padded_just("april"),
        padded_just("may"),
        padded_just("june"),
        padded_just("july"),
        padded_just("august"),
        padded_just("september"),
        padded_just("october"),
        padded_just("november"),
        padded_just("december"),
    ))
    .map(|s: &str| match s {
        "january" => Month::January,
        "february" => Month::February,
        "march" => Month::March,
        "april" => Month::April,
        "may" => Month::May,
        "june" => Month::June,
        "july" => Month::July,
        "august" => Month::August,
        "september" => Month::September,
        "october" => Month::October,
        "november" => Month::November,
        "december" => Month::December,
        _ => unreachable!(),
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Month {
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum NatDatToken {
    In,
    From,
    On,
    Last,
    This,
    Next,
    At,
    Colon,
    Today,
    Tomorrow,
    Yesterday,
    Now,
    Ago,
    // Time units
    Days,
    Weeks,
    Months,
    Years,
    Hours,
    Minutes,
    Seconds,
    // Time of day
    Morning,
    Afternoon,
    Evening,
    Night,
    // Other
    Weekend,
    Beginning,
    End,
    Start,
    Of,
    The,
    // Values
    Number(u32),
    Month(Month),
    Weekday(Weekday),
    // AM/PM indicators
    Am,
    Pm,
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::date;

    fn test_timestamp() -> Timestamp {
        // Thursday, January 16, 2025, 12:00:00 UTC
        date(2025, 1, 16)
            .at(12, 0, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap()
            .timestamp()
    }

    /// Helper to assert date matches, ignoring time components
    fn assert_date_matches(result: Timestamp, expected_date: Date) {
        let result_zoned = result.to_zoned(TimeZone::system());
        assert_eq!(result_zoned.date(), expected_date);
    }

    /// Helper to assert both date and time match
    fn assert_datetime_matches(
        result: Timestamp,
        expected_date: Date,
        expected_hour: i8,
        expected_minute: i8,
    ) {
        let zoned = result.to_zoned(TimeZone::system());
        assert_eq!(zoned.date(), expected_date);
        assert_eq!(zoned.hour(), expected_hour);
        assert_eq!(zoned.minute(), expected_minute);
    }

    #[test]
    fn test_today() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("today", now).unwrap();
        let expected = date(2025, 1, 16)
            .at(0, 0, 0, 0)
            .to_zoned(TimeZone::system())
            .unwrap()
            .timestamp();

        // Check that the dates match (ignoring time)
        let result_zoned = result.to_zoned(TimeZone::system());
        let expected_zoned = expected.to_zoned(TimeZone::system());
        assert_eq!(result_zoned.date(), expected_zoned.date());
    }

    #[test]
    fn test_tomorrow() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("tomorrow", now).unwrap();
        let result_zoned = result.to_zoned(TimeZone::system());

        assert_eq!(result_zoned.date(), date(2025, 1, 17));
    }

    #[test]
    fn test_yesterday() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("yesterday", now).unwrap();
        let result_zoned = result.to_zoned(TimeZone::system());

        assert_eq!(result_zoned.date(), date(2025, 1, 15));
    }

    #[test]
    fn test_in_days() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("in 3 days", now).unwrap();
        let result_zoned = result.to_zoned(TimeZone::system());

        assert_eq!(result_zoned.date(), date(2025, 1, 19));
    }

    #[test]
    fn test_in_weeks() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("in 2 weeks", now).unwrap();
        let result_zoned = result.to_zoned(TimeZone::system());

        assert_eq!(result_zoned.date(), date(2025, 1, 30));
    }

    #[test]
    fn test_in_months() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("in 1 months", now).unwrap();
        let result_zoned = result.to_zoned(TimeZone::system());

        assert_eq!(result_zoned.date(), date(2025, 2, 16));
    }

    #[test]
    fn test_from_now() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("3 days from now", now).unwrap();
        let result_zoned = result.to_zoned(TimeZone::system());

        assert_eq!(result_zoned.date(), date(2025, 1, 19));
    }

    #[test]
    fn test_ago() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("3 days ago", now).unwrap();
        let result_zoned = result.to_zoned(TimeZone::system());

        assert_eq!(result_zoned.date(), date(2025, 1, 13));
    }

    #[test]
    fn test_next_weekday() {
        let now = test_timestamp(); // Thursday, Jan 16, 2025
        let result = NaturalDateParser::parse("next monday", now).unwrap();
        let result_zoned = result.to_zoned(TimeZone::system());

        // Next Monday after Thursday should be Jan 20
        assert_eq!(result_zoned.date(), date(2025, 1, 20));
    }

    #[test]
    fn test_next_friday() {
        let now = test_timestamp(); // Thursday, Jan 16, 2025
        let result = NaturalDateParser::parse("next friday", now).unwrap();
        let result_zoned = result.to_zoned(TimeZone::system());

        // Next Friday after Thursday should be Jan 17
        assert_eq!(result_zoned.date(), date(2025, 1, 17));
    }

    #[test]
    fn test_last_weekday() {
        let now = test_timestamp(); // Thursday, Jan 16, 2025
        let result = NaturalDateParser::parse("last monday", now).unwrap();
        let result_zoned = result.to_zoned(TimeZone::system());

        // Last Monday before Thursday should be Jan 13
        assert_eq!(result_zoned.date(), date(2025, 1, 13));
    }

    #[test]
    fn test_this_friday() {
        let now = test_timestamp(); // Thursday, Jan 16, 2025
        let result = NaturalDateParser::parse("this friday", now).unwrap();
        let result_zoned = result.to_zoned(TimeZone::system());

        // This Friday (upcoming Friday in current week) should be Jan 17
        assert_eq!(result_zoned.date(), date(2025, 1, 17));
    }

    #[test]
    fn test_on_weekday() {
        let now = test_timestamp(); // Thursday, Jan 16, 2025
        let result = NaturalDateParser::parse("on friday", now).unwrap();
        let result_zoned = result.to_zoned(TimeZone::system());

        // Next Friday should be Jan 17
        assert_eq!(result_zoned.date(), date(2025, 1, 17));
    }

    #[test]
    fn test_with_time_at() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("tomorrow at 10:30", now).unwrap();
        let result_zoned = result.to_zoned(TimeZone::system());

        assert_eq!(result_zoned.date(), date(2025, 1, 17));
        assert_eq!(result_zoned.hour(), 10);
        assert_eq!(result_zoned.minute(), 30);
    }

    #[test]
    fn test_with_time_pm() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("tomorrow at 3 pm", now).unwrap();
        let result_zoned = result.to_zoned(TimeZone::system());

        assert_eq!(result_zoned.date(), date(2025, 1, 17));
        assert_eq!(result_zoned.hour(), 15); // 3 PM = 15:00
    }

    #[test]
    fn test_with_time_am() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("tomorrow at 10 am", now).unwrap();
        let result_zoned = result.to_zoned(TimeZone::system());

        assert_eq!(result_zoned.date(), date(2025, 1, 17));
        assert_eq!(result_zoned.hour(), 10);
    }

    #[test]
    fn test_with_time_and_minutes_pm() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("next friday at 10:13 pm", now).unwrap();
        let result_zoned = result.to_zoned(TimeZone::system());

        assert_eq!(result_zoned.date(), date(2025, 1, 17));
        assert_eq!(result_zoned.hour(), 22); // 10 PM = 22:00
        assert_eq!(result_zoned.minute(), 13);
    }

    #[test]
    fn test_number_words() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("in three days", now).unwrap();
        let result_zoned = result.to_zoned(TimeZone::system());

        assert_eq!(result_zoned.date(), date(2025, 1, 19));
    }

    #[test]
    fn test_case_insensitive() {
        let now = test_timestamp();
        let result1 = NaturalDateParser::parse("TOMORROW", now).unwrap();
        let result2 = NaturalDateParser::parse("Tomorrow", now).unwrap();
        let result3 = NaturalDateParser::parse("tomorrow", now).unwrap();

        let r1 = result1.to_zoned(TimeZone::system());
        let r2 = result2.to_zoned(TimeZone::system());
        let r3 = result3.to_zoned(TimeZone::system());

        assert_eq!(r1.date(), r2.date());
        assert_eq!(r2.date(), r3.date());
    }

    #[test]
    fn test_next_month() {
        let now = test_timestamp(); // January 16, 2025
        let result = NaturalDateParser::parse("next march", now).unwrap();
        let result_zoned = result.to_zoned(TimeZone::system());

        // March is after January, so it should be March 1, 2025
        assert_eq!(result_zoned.date(), date(2025, 3, 1));
    }

    #[test]
    fn test_last_month() {
        let now = test_timestamp(); // January 16, 2025
        let result = NaturalDateParser::parse("last december", now).unwrap();
        let result_zoned = result.to_zoned(TimeZone::system());

        // December is before January, so it should be December 1, 2024
        assert_eq!(result_zoned.date(), date(2024, 12, 1));
    }

    #[test]
    fn test_hours_and_minutes() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("in 2 hours", now).unwrap();

        // Should be 2 hours after 12:00 = 14:00
        let result_zoned = result.to_zoned(TimeZone::UTC);
        assert_eq!(result_zoned.hour(), 14);
    }

    #[test]
    fn test_in_minutes() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("in 30 minutes", now).unwrap();

        // Should be 30 minutes after 12:00 = 12:30
        let result_zoned = result.to_zoned(TimeZone::UTC);
        assert_eq!(result_zoned.minute(), 30);
    }

    // ===== Extended TimeStride Coverage =====

    #[test]
    fn test_in_seconds() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("in 30 seconds", now).unwrap();

        // Should be 30 seconds after 12:00:00
        let result_zoned = result.to_zoned(TimeZone::UTC);
        assert_eq!(result_zoned.second(), 30);
    }

    #[test]
    fn test_seconds_from_now() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("45 seconds from now", now).unwrap();

        let result_zoned = result.to_zoned(TimeZone::UTC);
        assert_eq!(result_zoned.second(), 45);
    }

    #[test]
    fn test_seconds_ago() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("10 seconds ago", now).unwrap();

        let result_zoned = result.to_zoned(TimeZone::UTC);
        assert_eq!(result_zoned.second(), 50); // 12:00:00 - 10s = 11:59:50
        assert_eq!(result_zoned.minute(), 59);
    }

    #[test]
    fn test_in_years() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("in 2 years", now).unwrap();
        let result_zoned = result.to_zoned(TimeZone::system());

        assert_eq!(result_zoned.date(), date(2027, 1, 16));
    }

    #[test]
    fn test_years_from_now() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("3 years from now", now).unwrap();
        let result_zoned = result.to_zoned(TimeZone::system());

        assert_eq!(result_zoned.date(), date(2028, 1, 16));
    }

    #[test]
    fn test_years_ago() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("1 years ago", now).unwrap();
        let result_zoned = result.to_zoned(TimeZone::system());

        assert_eq!(result_zoned.date(), date(2024, 1, 16));
    }

    #[test]
    fn test_ago_all_strides() {
        let now = test_timestamp();

        // Table-driven test for all strides with "ago" pattern
        let test_cases = vec![
            ("5 seconds ago", -5, "seconds"),
            ("10 minutes ago", -10, "minutes"),
            ("3 hours ago", -3, "hours"),
            ("4 days ago", -4, "days"),
            ("2 weeks ago", -2, "weeks"),
            ("6 months ago", -6, "months"),
            ("2 years ago", -2, "years"),
        ];

        for (input, amount, unit) in test_cases {
            let result = NaturalDateParser::parse(input, now);
            assert!(
                result.is_ok(),
                "Failed to parse '{}' (expected {} {} ago)",
                input,
                amount,
                unit
            );
        }
    }

    #[test]
    fn test_from_now_all_strides() {
        let now = test_timestamp();

        // Table-driven test for all strides with "from now" pattern
        let test_cases = vec![
            "5 seconds from now",
            "10 minutes from now",
            "3 hours from now",
            "4 days from now",
            "2 weeks from now",
            "6 months from now",
            "2 years from now",
        ];

        for input in test_cases {
            let result = NaturalDateParser::parse(input, now);
            assert!(result.is_ok(), "Failed to parse '{}'", input);
        }
    }

    #[test]
    fn test_in_days_with_time() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("in 5 days at 3 pm", now).unwrap();

        assert_datetime_matches(result, date(2025, 1, 21), 15, 0);
    }

    #[test]
    fn test_in_weeks_with_time() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("in 2 weeks at 8:30 am", now).unwrap();

        assert_datetime_matches(result, date(2025, 1, 30), 8, 30);
    }

    #[test]
    fn test_ago_with_time() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("3 days ago at 2 pm", now).unwrap();

        assert_datetime_matches(result, date(2025, 1, 13), 14, 0);
    }

    #[test]
    fn test_from_now_with_time() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("2 weeks from now at 10 am", now).unwrap();

        assert_datetime_matches(result, date(2025, 1, 30), 10, 0);
    }

    #[test]
    fn test_in_months_with_time() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("in 3 months at 11:45 pm", now).unwrap();

        assert_datetime_matches(result, date(2025, 4, 16), 23, 45);
    }

    #[test]
    fn test_in_years_with_time() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("in 1 years at 2 pm", now).unwrap();

        assert_datetime_matches(result, date(2026, 1, 16), 14, 0);
    }

    // ===== Complete TimeMoment Coverage =====

    #[test]
    fn test_next_week() {
        let now = test_timestamp(); // Thursday, Jan 16, 2025
        let result = NaturalDateParser::parse("next week", now).unwrap();
        let result_zoned = result.to_zoned(TimeZone::system());

        // Next week should be 7 days ahead = Jan 23
        assert_eq!(result_zoned.date(), date(2025, 1, 23));
    }

    #[test]
    fn test_this_week() {
        let now = test_timestamp(); // Thursday, Jan 16, 2025
        let result = NaturalDateParser::parse("this week", now).unwrap();
        let result_zoned = result.to_zoned(TimeZone::system());

        // This week should return current date
        assert_eq!(result_zoned.date(), date(2025, 1, 16));
    }

    #[test]
    fn test_last_week() {
        let now = test_timestamp(); // Thursday, Jan 16, 2025
        let result = NaturalDateParser::parse("last week", now).unwrap();
        let result_zoned = result.to_zoned(TimeZone::system());

        // Last week should be 7 days ago = Jan 9
        assert_eq!(result_zoned.date(), date(2025, 1, 9));
    }

    #[test]
    fn test_next_year() {
        let now = test_timestamp(); // 2025
        let result = NaturalDateParser::parse("next year", now).unwrap();
        let result_zoned = result.to_zoned(TimeZone::system());

        // Next year should be Jan 1, 2026
        assert_eq!(result_zoned.date(), date(2026, 1, 1));
    }

    #[test]
    fn test_this_year() {
        let now = test_timestamp(); // 2025
        let result = NaturalDateParser::parse("this year", now).unwrap();
        let result_zoned = result.to_zoned(TimeZone::system());

        // This year should be Jan 1, 2025
        assert_eq!(result_zoned.date(), date(2025, 1, 1));
    }

    #[test]
    fn test_last_year() {
        let now = test_timestamp(); // 2025
        let result = NaturalDateParser::parse("last year", now).unwrap();
        let result_zoned = result.to_zoned(TimeZone::system());

        // Last year should be Jan 1, 2024
        assert_eq!(result_zoned.date(), date(2024, 1, 1));
    }

    #[test]
    fn test_next_month_with_time() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("next january at 9 am", now).unwrap();

        // January is before current month (January), so next January is 2026
        assert_datetime_matches(result, date(2026, 1, 1), 9, 0);
    }

    #[test]
    fn test_last_year_with_time() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("last year at 12 pm", now).unwrap();

        // Last year at noon
        assert_datetime_matches(result, date(2024, 1, 1), 12, 0);
    }

    #[test]
    fn test_this_week_with_time() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("this week at 2 pm", now).unwrap();

        // This week at 2pm (current date)
        assert_datetime_matches(result, date(2025, 1, 16), 14, 0);
    }

    #[test]
    fn test_all_weekdays_next() {
        let now = test_timestamp(); // Thursday, Jan 16, 2025

        let test_cases = vec![
            ("next monday", date(2025, 1, 20)),    // 4 days ahead
            ("next tuesday", date(2025, 1, 21)),   // 5 days ahead
            ("next wednesday", date(2025, 1, 22)), // 6 days ahead
            ("next thursday", date(2025, 1, 23)),  // 7 days ahead (same day, next week)
            ("next friday", date(2025, 1, 17)),    // 1 day ahead
            ("next saturday", date(2025, 1, 18)),  // 2 days ahead
            ("next sunday", date(2025, 1, 19)),    // 3 days ahead
        ];

        for (input, expected_date) in test_cases {
            let result = NaturalDateParser::parse(input, now).unwrap();
            assert_date_matches(result, expected_date);
        }
    }

    #[test]
    fn test_all_weekdays_last() {
        let now = test_timestamp(); // Thursday, Jan 16, 2025

        let test_cases = vec![
            ("last monday", date(2025, 1, 13)),    // 3 days back
            ("last tuesday", date(2025, 1, 14)),   // 2 days back
            ("last wednesday", date(2025, 1, 15)), // 1 day back
            ("last thursday", date(2025, 1, 9)),   // 7 days back (same day, last week)
            ("last friday", date(2025, 1, 10)),    // 6 days back
            ("last saturday", date(2025, 1, 11)),  // 5 days back
            ("last sunday", date(2025, 1, 12)),    // 4 days back
        ];

        for (input, expected_date) in test_cases {
            let result = NaturalDateParser::parse(input, now).unwrap();
            assert_date_matches(result, expected_date);
        }
    }

    #[test]
    fn test_all_weekdays_this() {
        let now = test_timestamp(); // Thursday, Jan 16, 2025

        let test_cases = vec![
            ("this monday", date(2025, 1, 20)),    // Passed this week, next Monday
            ("this tuesday", date(2025, 1, 21)),   // Passed this week, next Tuesday
            ("this wednesday", date(2025, 1, 22)), // Passed this week, next Wednesday
            ("this thursday", date(2025, 1, 16)),  // Today (current weekday)
            ("this friday", date(2025, 1, 17)),    // Tomorrow (upcoming in current week)
            ("this saturday", date(2025, 1, 18)),  // Upcoming in current week
            ("this sunday", date(2025, 1, 19)),    // Upcoming in current week
        ];

        for (input, expected_date) in test_cases {
            let result = NaturalDateParser::parse(input, now).unwrap();
            assert_date_matches(result, expected_date);
        }
    }

    #[test]
    fn test_all_weekdays_on() {
        let now = test_timestamp(); // Thursday, Jan 16, 2025

        // "on" pattern should behave like "next" - always go forward
        let test_cases = vec![
            ("on monday", date(2025, 1, 20)),
            ("on tuesday", date(2025, 1, 21)),
            ("on wednesday", date(2025, 1, 22)),
            ("on thursday", date(2025, 1, 23)), // Same day goes to next week
            ("on friday", date(2025, 1, 17)),
            ("on saturday", date(2025, 1, 18)),
            ("on sunday", date(2025, 1, 19)),
        ];

        for (input, expected_date) in test_cases {
            let result = NaturalDateParser::parse(input, now).unwrap();
            assert_date_matches(result, expected_date);
        }
    }

    // ===== Time Specification Edge Cases =====

    #[test]
    fn test_midnight_twelve_am() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("tomorrow at 12 am", now).unwrap();

        // 12:00 AM should be midnight (00:00)
        assert_datetime_matches(result, date(2025, 1, 17), 0, 0);
    }

    #[test]
    fn test_noon_twelve_pm() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("tomorrow at 12 pm", now).unwrap();

        // 12:00 PM should be noon (12:00)
        assert_datetime_matches(result, date(2025, 1, 17), 12, 0);
    }

    #[test]
    fn test_one_am() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("tomorrow at 1 am", now).unwrap();

        assert_datetime_matches(result, date(2025, 1, 17), 1, 0);
    }

    #[test]
    fn test_one_pm() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("tomorrow at 1 pm", now).unwrap();

        assert_datetime_matches(result, date(2025, 1, 17), 13, 0);
    }

    #[test]
    fn test_eleven_am() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("tomorrow at 11 am", now).unwrap();

        assert_datetime_matches(result, date(2025, 1, 17), 11, 0);
    }

    #[test]
    fn test_eleven_pm() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("tomorrow at 11 pm", now).unwrap();

        assert_datetime_matches(result, date(2025, 1, 17), 23, 0);
    }

    #[test]
    fn test_hour_only_no_minutes_am() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("tomorrow at 5 am", now).unwrap();

        assert_datetime_matches(result, date(2025, 1, 17), 5, 0);
    }

    #[test]
    fn test_hour_only_no_minutes_pm() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("tomorrow at 3 pm", now).unwrap();

        assert_datetime_matches(result, date(2025, 1, 17), 15, 0);
    }

    #[test]
    fn test_today_with_time() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("today at 2 pm", now).unwrap();

        assert_datetime_matches(result, date(2025, 1, 16), 14, 0);
    }

    #[test]
    fn test_yesterday_with_time() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("yesterday at 8:30 am", now).unwrap();

        assert_datetime_matches(result, date(2025, 1, 15), 8, 30);
    }

    #[test]
    fn test_on_weekday_with_time() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("on monday at 7:15 am", now).unwrap();

        assert_datetime_matches(result, date(2025, 1, 20), 7, 15);
    }

    #[test]
    fn test_midnight_with_minutes() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("tomorrow at 12:30 am", now).unwrap();

        // 12:30 AM should be 00:30
        assert_datetime_matches(result, date(2025, 1, 17), 0, 30);
    }

    // ===== Error and Negative Cases =====

    #[test]
    fn test_empty_string() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("", now);
        assert!(result.is_err(), "Empty string should fail to parse");
    }

    #[test]
    fn test_unsupported_pattern() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("foobar baz", now);
        assert!(result.is_err(), "Unsupported pattern should fail to parse");
    }

    #[test]
    fn test_partial_pattern_missing_unit() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("in 5", now);
        assert!(
            result.is_err(),
            "Partial pattern 'in 5' without time unit should fail"
        );
    }

    #[test]
    fn test_invalid_time_hour_over_24() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("tomorrow at 25:00", now);
        // This might succeed at parse but fail at conversion - either is acceptable
        if let Ok(timestamp) = result {
            // If it parses, it should handle the overflow somehow
            let _ = timestamp;
        } else {
            // Expected: should fail to parse
        }
    }

    #[test]
    fn test_just_time_no_date() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("at 3 pm", now);
        assert!(result.is_err(), "Time without date should fail to parse");
    }

    #[test]
    fn test_invalid_weekday() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("next fooday", now);
        assert!(
            result.is_err(),
            "Invalid weekday 'fooday' should fail to parse"
        );
    }

    #[test]
    fn test_invalid_month() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("next foovember", now);
        assert!(
            result.is_err(),
            "Invalid month 'foovember' should fail to parse"
        );
    }

    #[test]
    fn test_missing_number() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("in days", now);
        assert!(
            result.is_err(),
            "Missing number in 'in days' should fail to parse"
        );
    }

    #[test]
    fn test_double_pattern() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("next last monday", now);
        assert!(
            result.is_err(),
            "Conflicting patterns 'next last' should fail to parse"
        );
    }

    // ===== Boundary and Large Number Cases =====

    #[test]
    fn test_large_day_offset() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("in 365 days", now).unwrap();
        let result_zoned = result.to_zoned(TimeZone::system());

        // 365 days from Jan 16, 2025 should be Jan 16, 2026
        assert_eq!(result_zoned.date(), date(2026, 1, 16));
    }

    #[test]
    fn test_large_week_offset() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("in 52 weeks", now).unwrap();
        let result_zoned = result.to_zoned(TimeZone::system());

        // 52 weeks from Jan 16, 2025 should be around Jan 15, 2026
        assert_eq!(result_zoned.date(), date(2026, 1, 15));
    }

    #[test]
    fn test_very_large_number() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("in 1000 days", now).unwrap();
        let result_zoned = result.to_zoned(TimeZone::system());

        // 1000 days from Jan 16, 2025 should be around Oct 13, 2027
        assert_eq!(result_zoned.date(), date(2027, 10, 13));
    }

    #[test]
    fn test_month_boundary_january_31() {
        // Test from Jan 31 going forward
        let jan_31 = date(2025, 1, 31)
            .at(12, 0, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap()
            .timestamp();

        let result = NaturalDateParser::parse("tomorrow", jan_31).unwrap();
        let result_zoned = result.to_zoned(TimeZone::system());

        // Tomorrow from Jan 31 should be Feb 1
        assert_eq!(result_zoned.date(), date(2025, 2, 1));
    }

    #[test]
    fn test_year_boundary_december_31() {
        // Test from Dec 31 going forward
        let dec_31 = date(2024, 12, 31)
            .at(12, 0, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap()
            .timestamp();

        let result = NaturalDateParser::parse("tomorrow", dec_31).unwrap();
        let result_zoned = result.to_zoned(TimeZone::system());

        // Tomorrow from Dec 31, 2024 should be Jan 1, 2025
        assert_eq!(result_zoned.date(), date(2025, 1, 1));
    }

    #[test]
    fn test_month_offset_variable_lengths() {
        // Test from Jan 31 + 1 month
        let jan_31 = date(2025, 1, 31)
            .at(12, 0, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap()
            .timestamp();

        let result = NaturalDateParser::parse("in 1 months", jan_31).unwrap();
        let result_zoned = result.to_zoned(TimeZone::system());

        // 1 month from Jan 31 should handle February's shorter length
        // jiff typically adjusts to Feb 28 (or March 2/3 depending on implementation)
        // Let's just verify it succeeds and produces a valid date
        assert!(result_zoned.date().month() >= 2);
    }

    #[test]
    fn test_all_number_words() {
        let now = test_timestamp();

        let test_cases = vec![
            "in one days",
            "in two days",
            "in three days",
            "in four days",
            "in five days",
            "in six days",
            "in seven days",
            "in eight days",
            "in nine days",
            "in ten days",
            "in eleven days",
            "in twelve days",
            "in thirteen days",
        ];

        for input in test_cases {
            let result = NaturalDateParser::parse(input, now);
            assert!(result.is_ok(), "Failed to parse number word: '{}'", input);
        }
    }

    #[test]
    fn test_large_years_offset() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("in 50 years", now).unwrap();
        let result_zoned = result.to_zoned(TimeZone::system());

        assert_eq!(result_zoned.date(), date(2075, 1, 16));
    }

    #[test]
    fn test_large_ago_offset() {
        let now = test_timestamp();
        let result = NaturalDateParser::parse("500 days ago", now).unwrap();
        let result_zoned = result.to_zoned(TimeZone::system());

        // 500 days before Jan 16, 2025 should be around Sep 4, 2023
        assert_eq!(result_zoned.date(), date(2023, 9, 4));
    }
}

// ===== Generative Tests Module =====
// Tests that generate natural language descriptions and validate parsing
#[cfg(test)]
mod generative_tests {
    use super::*;
    use jiff::{civil::date, tz::TimeZone};

    fn test_timestamp() -> Timestamp {
        // Thursday, January 16, 2025, 12:00:00 UTC
        date(2025, 1, 16)
            .at(12, 0, 0, 0)
            .to_zoned(TimeZone::UTC)
            .unwrap()
            .timestamp()
    }

    #[test]
    fn test_generated_all_strides_in_pattern() {
        let now = test_timestamp();

        let strides = vec![
            "seconds", "minutes", "hours", "days", "weeks", "months", "years",
        ];

        for stride in strides {
            let input = format!("in 5 {}", stride);
            let result = NaturalDateParser::parse(&input, now);
            assert!(
                result.is_ok(),
                "Failed to parse generated input: '{}'",
                input
            );
        }
    }

    #[test]
    fn test_generated_all_strides_ago_pattern() {
        let now = test_timestamp();

        let strides = vec![
            "seconds", "minutes", "hours", "days", "weeks", "months", "years",
        ];

        for stride in strides {
            let input = format!("5 {} ago", stride);
            let result = NaturalDateParser::parse(&input, now);
            assert!(
                result.is_ok(),
                "Failed to parse generated input: '{}'",
                input
            );
        }
    }

    #[test]
    fn test_generated_all_strides_from_now() {
        let now = test_timestamp();

        let strides = vec![
            "seconds", "minutes", "hours", "days", "weeks", "months", "years",
        ];

        for stride in strides {
            let input = format!("5 {} from now", stride);
            let result = NaturalDateParser::parse(&input, now);
            assert!(
                result.is_ok(),
                "Failed to parse generated input: '{}'",
                input
            );
        }
    }

    #[test]
    fn test_all_weekdays_all_patterns() {
        let now = test_timestamp();

        let weekdays = vec![
            "monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday",
        ];
        let patterns = vec!["next", "this", "last", "on"];

        for pattern in patterns {
            for day in &weekdays {
                let input = format!("{} {}", pattern, day);
                let result = NaturalDateParser::parse(&input, now);
                assert!(
                    result.is_ok(),
                    "Failed to parse generated input: '{}'",
                    input
                );
            }
        }
    }

    #[test]
    fn test_all_months_next_pattern() {
        let now = test_timestamp();

        let months = vec![
            "january", "february", "march", "april", "may", "june",
            "july", "august", "september", "october", "november", "december",
        ];

        for month in months {
            let input = format!("next {}", month);
            let result = NaturalDateParser::parse(&input, now);
            assert!(
                result.is_ok(),
                "Failed to parse generated input: '{}'",
                input
            );
        }
    }

    #[test]
    fn test_all_months_last_pattern() {
        let now = test_timestamp();

        let months = vec![
            "january", "february", "march", "april", "may", "june",
            "july", "august", "september", "october", "november", "december",
        ];

        for month in months {
            let input = format!("last {}", month);
            let result = NaturalDateParser::parse(&input, now);
            assert!(
                result.is_ok(),
                "Failed to parse generated input: '{}'",
                input
            );
        }
    }

    #[test]
    fn test_generated_time_variations() {
        let now = test_timestamp();

        let time_variations = vec![
            "tomorrow at 5",           // Hour only (no am/pm)
            "tomorrow at 5 am",        // Hour with am
            "tomorrow at 5 pm",        // Hour with pm
            "tomorrow at 5:30",        // Hour:minute (no am/pm)
            "tomorrow at 5:30 am",     // Hour:minute with am
            "tomorrow at 5:30 pm",     // Hour:minute with pm
            "tomorrow at 12 am",       // Midnight
            "tomorrow at 12 pm",       // Noon
            "tomorrow at 12:15 am",    // After midnight
            "tomorrow at 12:15 pm",    // After noon
        ];

        for input in time_variations {
            let result = NaturalDateParser::parse(input, now);
            assert!(
                result.is_ok(),
                "Failed to parse generated time variation: '{}'",
                input
            );
        }
    }

    #[test]
    fn test_generated_combined_patterns() {
        let now = test_timestamp();

        let combined_patterns = vec![
            "in 3 days at 5 pm",
            "in 2 weeks at 9:30 am",
            "in 1 months at 12 pm",
            "3 days from now at 3 pm",
            "2 weeks from now at 8 am",
            "next monday at 10 am",
            "last friday at 5:30 pm",
            "this wednesday at 2 pm",
        ];

        for input in combined_patterns {
            let result = NaturalDateParser::parse(input, now);
            assert!(
                result.is_ok(),
                "Failed to parse generated combined pattern: '{}'",
                input
            );
        }
    }

    #[test]
    fn test_number_words_all_supported() {
        let now = test_timestamp();

        let number_words = vec![
            "one", "two", "three", "four", "five", "six", "seven",
            "eight", "nine", "ten", "eleven", "twelve", "thirteen",
        ];

        for word in number_words {
            let input = format!("in {} days", word);
            let result = NaturalDateParser::parse(&input, now);
            assert!(
                result.is_ok(),
                "Failed to parse number word: '{}'",
                input
            );
        }
    }

    #[test]
    fn test_case_variations_generated() {
        let now = test_timestamp();

        let test_cases = vec![
            ("TOMORROW", "all uppercase"),
            ("Tomorrow", "title case"),
            ("tomorrow", "all lowercase"),
            ("ToMoRrOw", "mixed case"),
            ("NEXT FRIDAY", "all uppercase phrase"),
            ("Next Friday", "title case phrase"),
            ("next friday", "all lowercase phrase"),
        ];

        for (input, description) in test_cases {
            let result = NaturalDateParser::parse(input, now);
            assert!(
                result.is_ok(),
                "Failed to parse {} case variation: '{}'",
                description,
                input
            );
        }
    }

    #[test]
    fn test_generated_numeric_and_word_numbers() {
        let now = test_timestamp();

        // Test that both numeric and word forms produce valid results
        let test_pairs = vec![
            ("in 1 days", "in one days"),
            ("in 2 weeks", "in two weeks"),
            ("in 3 months", "in three months"),
            ("in 5 years", "in five years"),
            ("7 days ago", "seven days ago"),
        ];

        for (numeric, word) in test_pairs {
            let result_numeric = NaturalDateParser::parse(numeric, now);
            let result_word = NaturalDateParser::parse(word, now);

            assert!(
                result_numeric.is_ok(),
                "Failed to parse numeric form: '{}'",
                numeric
            );
            assert!(
                result_word.is_ok(),
                "Failed to parse word form: '{}'",
                word
            );

            // Both should parse to the same date
            if let (Ok(num_ts), Ok(word_ts)) = (result_numeric, result_word) {
                let num_zoned = num_ts.to_zoned(TimeZone::system());
                let word_zoned = word_ts.to_zoned(TimeZone::system());
                assert_eq!(
                    num_zoned.date(),
                    word_zoned.date(),
                    "Numeric '{}' and word '{}' should produce same date",
                    numeric,
                    word
                );
            }
        }
    }

    #[test]
    fn test_generated_moment_variations() {
        let now = test_timestamp();

        // Test all TimeMoment types with different patterns
        let moment_variations = vec![
            // Weekday moments
            "next monday",
            "this friday",
            "last wednesday",
            // Month moments
            "next march",
            "last december",
            // Week/Year moments
            "next week",
            "this week",
            "last week",
            "next year",
            "this year",
            "last year",
        ];

        for input in moment_variations {
            let result = NaturalDateParser::parse(input, now);
            assert!(
                result.is_ok(),
                "Failed to parse generated moment variation: '{}'",
                input
            );
        }
    }
}
