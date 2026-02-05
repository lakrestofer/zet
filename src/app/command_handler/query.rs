use std::io::BufWriter;
use std::io::Write;
use std::ops::Range;

use jiff::Timestamp;
use pulldown_cmark::Parser;
use zet::core::db::DB;
use zet::core::db::DbList;
use zet::core::parser::DocumentParser;
use zet::core::parser::FrontMatterFormat;
use zet::core::parser::FrontMatterParser;
use zet::core::types::document::Document;

use crate::app::commands::MatchStrategy;
use crate::app::commands::OutputFormat;
use crate::app::commands::SortByOption;
use crate::app::commands::SortConfig;
use crate::app::commands::SortOrder;
use crate::app::preamble::*;
use zet::preamble::*;

/// Instead of producing an ast, this command simply outputs the event stream as
/// returned by pulldown_cmark
pub fn handle_command(
    config: zet::config::Config,
    tags: Vec<String>,
    tagless: bool,

    exclude_list: Vec<String>,
    exclude_by_path: Vec<String>,
    created: Option<Timestamp>,
    modified: Option<Timestamp>,
    created_before: Option<Timestamp>,
    created_after: Option<Timestamp>,
    modified_before: Option<Timestamp>,
    modified_after: Option<Timestamp>,
    links_to: Vec<String>,
    links_from: Vec<String>,
    match_pattern: Vec<String>,
    match_strategy: MatchStrategy,
    sort_configs: Vec<SortConfig>,
    limit: Option<usize>,
    output_format: OutputFormat,
    delimiter: Option<String>,
    pretty: bool,
    template: Option<String>,
) -> Result<()> {
    let root = &config.root;
    let db_path = zet::core::db_dir(root);
    let mut db = DB::open(db_path)?;

    let separator = delimiter.unwrap_or("\n".into());

    let mut documents = Document::list(&mut db)?;

    // sort by options
    sort_documents_by_config(sort_configs, &mut documents);

    // limit total lines rendered
    if let Some(limit) = limit {
        documents.truncate(limit);
    };

    let writer = std::io::BufWriter::new(std::io::stdout());
    render_query_output(output_format, pretty, separator, documents, writer)?;

    Ok(())
}

fn render_query_output(
    output_format: OutputFormat,
    pretty: bool,
    separator: String,
    documents: Vec<Document>,
    mut writer: BufWriter<std::io::Stdout>,
) -> Result<()> {
    match output_format {
        OutputFormat::Ids => {
            for d in documents {
                write!(writer, "{}{separator}", d.id.0)?;
            }
        }
        OutputFormat::Json => {
            if pretty {
                serde_json::to_writer_pretty(writer, &documents)?;
            } else {
                serde_json::to_writer(writer, &documents)?;
            }
        }
        OutputFormat::Template => {
            todo!()
        }
    }
    Ok(())
}

fn sort_documents_by_config(sort_configs: Vec<SortConfig>, documents: &mut Vec<Document>) {
    documents.sort_by(|a, b| {
        for SortConfig { by, order } in &sort_configs {
            let res = match by {
                SortByOption::Modified => a.modified.cmp(&b.modified),
                SortByOption::Created => a.created.cmp(&b.created),
                SortByOption::Id => a.id.cmp(&b.id),
                SortByOption::Path => a.path.cmp(&b.path),
                SortByOption::Title => a.title.cmp(&b.title),
            };
            let res = match order {
                SortOrder::Ascending => res,
                SortOrder::Descending => res.reverse(),
            };
            match res {
                std::cmp::Ordering::Equal => {}
                _ => return res,
            }
        }
        std::cmp::Ordering::Equal
    });
}
