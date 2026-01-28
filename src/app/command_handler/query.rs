use std::io::BufWriter;
use std::io::Write;
use std::ops::Range;

use jiff::Timestamp;
use pulldown_cmark::Parser;
use zet::core::parser::DocumentParser;
use zet::core::parser::FrontMatterFormat;
use zet::core::parser::FrontMatterParser;

use crate::app::commands::MatchStrategy;
use crate::app::preamble::*;
use zet::preamble::*;

/// Instead of producing an ast, this command simply outputs the event stream as
/// returned by pulldown_cmark
pub fn handle_command(
    // tags
    tag: Vec<String>,

    // explicitly exclude some document ids from the result set
    exclude: Vec<String>,

    // date range filtering
    created_before: Option<Timestamp>,
    created_after: Option<Timestamp>,
    modified_before: Option<Timestamp>,
    modified_after: Option<Timestamp>,

    // links
    link_to: Vec<String>,
    link_from: Vec<String>,

    // some match
    match_pattern: Vec<String>,
    match_strategy: MatchStrategy,

    format: Option<String>,
) -> Result<()> {
    Ok(())
}
