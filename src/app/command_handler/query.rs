use std::fmt::Debug;
use std::io::BufWriter;
use std::io::Write;
use std::ops::Range;

use jiff::Timestamp;
use pulldown_cmark::Parser;
use tera::Context;
use tera::Tera;
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

pub fn handle_command(
    // configuration context
    config: zet::config::Config,
    // query parameters
    ids: Vec<String>,
    titles: Vec<String>,
    paths: Vec<String>,
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

    let mut writer = std::io::BufWriter::new(std::io::stdout());
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
            let mut tera = Tera::default();
            tera.add_raw_template(DEFAULT_TEMPLATE_NAME, DEFAULT_TEMPLATE)?;

            match template {
                Some(template) => {
                    tera.add_raw_template(USER_INPUT_TEMPLATE_NAME, &template)?;

                    for d in documents {
                        let ctx = Context::from_serialize(d)?;
                        tera.render_to(USER_INPUT_TEMPLATE_NAME, &ctx, &mut writer)?;
                    }
                }
                None => {
                    for d in documents {
                        let ctx = Context::from_serialize(d)?;
                        tera.render_to(DEFAULT_TEMPLATE_NAME, &ctx, &mut writer)?;
                    }
                }
            }
        }
    }

    Ok(())
}

const USER_INPUT_TEMPLATE_NAME: &str = "user_input_template";
const DEFAULT_TEMPLATE: &str = r#"# {{ title }}\n"#;
const DEFAULT_TEMPLATE_NAME: &str = "default_template";

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
