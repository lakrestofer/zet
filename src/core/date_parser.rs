use chumsky::prelude::*;
use jiff::Timestamp;

pub struct NaturalDateParser;

// fn parser<'src>(now_t: Timestamp) -> impl Parser<'src, &'src str, Timestamp> {
//     let r#in = just("in").padded();

//     let day = just("day").then(just("s").or_not()).ignored();

//     // let number = choice((
//     //     just("one").to(1),
//     //     just("two").to(2),
//     //     just("three").to(3),
//     //     just("four").to(4),
//     //     just("five").to(5),
//     //     just("six").to(6),
//     //     just("seven").to(7),
//     //     just("eight").to(8),
//     //     just("nine").to(9),
//     //     just("ten").to(10),
//     //     just("eleven").to(11),
//     //     just("twelve").to(12),
//     //     just("thirteen").to(13),
//     //     text::int(10).map(|s: &str| s.parse().unwrap()),
//     // ))
//     // .padded();

//     // let duration = choice((
//     //     choice((just("now"), just("today")))
//     //         .padded()
//     //         .map(|_| jiff::SignedDuration::new(0, 0)),
//     //     just("in").padded().then(number),
//     // ))
//     // .then(end());

//     // now_t

//     todo!()
// }

// /// `at` `time`
// fn time_suffix<'src>() -> impl Parser<'src, &'src str, (u32, u32)> {
//     let at = just("at").padded();

//     let number = text::int(10).map(|s: &str| s.parse().unwrap() as u32);

//     let hour = number;
//     let minute = just(':').then(number).or_not();

//     let am_pm = choice((just("pm"), just("am"))).padded();
// }

// fn natural_lang_parser<'src>(now: Timestamp) -> impl Parser<'src, &'src[NatDatToken], Timestamp>

struct Time {
    hour: u32,
    minute: Option<u32>,
}

/// This enum tries to list some common natural language patterns for referring
/// to different moments in time
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
    /// on monday
    /// on friday at 07:15am
    OnWeekday { moment: Weekday, at: Option<Time> },
}

enum TimeMoment {
    Weekday(Weekday),
    Month(Month),
    Year,
}

enum TimeStride {
    Days,
    Weeks,
    Months,
    Years,
}

fn token_parser<'src>() -> impl Parser<'src, &'src str, Vec<NatDatToken>> {
    choice((
        just("in").padded().to(NatDatToken::In),
        just("from").padded().to(NatDatToken::From),
        just("on").padded().to(NatDatToken::On),
        just("last").padded().to(NatDatToken::Last),
        just(":").padded().to(NatDatToken::Colon),
        just("today").padded().to(NatDatToken::Today),
        just("tomorrow").padded().to(NatDatToken::Tomorrow),
        just("yesterday").padded().to(NatDatToken::Yesterday),
        just("this").padded().to(NatDatToken::This),
        choice((just("at").padded(), just("@").padded())).to(NatDatToken::At),
        month().map(|v| NatDatToken::Month(v)),
        weekday().map(|v| NatDatToken::Weekday(v)),
        number().map(|n| NatDatToken::Number(n)),
    ))
    .repeated()
    .collect()
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
        padded_just("noveber"),
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
        "noveber" => Month::Noveber,
        "december" => Month::December,
        _ => unreachable!(),
    })
}

#[derive(Clone, Copy)]
enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}
#[derive(Clone, Copy)]
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
    Noveber,
    December,
}

#[derive(Clone, Copy)]
enum NatDatToken {
    In,
    From,
    On,
    Last,
    This,
    At,
    Colon,
    Today,
    Tomorrow,
    Yesterday,
    Number(u32),
    Month(Month),
    Weekday(Weekday),
}
