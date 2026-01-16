// Example demonstrating the natural language date parser
use zet::core::date_parser::NaturalDateParser;
use jiff::Timestamp;

fn main() {
    let now = Timestamp::now();

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
                println!("{:25} -> {}", example, timestamp);
            }
            Err(e) => {
                println!("{:25} -> ERROR: {:?}", example, e);
            }
        }
    }
}
