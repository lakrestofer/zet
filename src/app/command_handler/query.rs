use std::io::Write;
use std::path::Path;

use jiff::Timestamp;
use tera::Context;
use tera::Tera;
use zet::core::db::DB;
use zet::core::query::DocumentQuery;
use zet::core::query::SortByOption as QuerySortByOption;
use zet::core::query::SortOrder as QuerySortOrder;

use crate::app::commands::MatchStrategy;
use crate::app::commands::OutputFormat;
use crate::app::commands::SortByOption;
use crate::app::commands::SortConfig;
use crate::app::commands::SortOrder;
use zet::preamble::*;

#[allow(clippy::too_many_arguments)]
pub fn handle_command(
    root: &Path,
    // configuration context
    _config: zet::config::Config,
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
    _match_pattern: Vec<String>,
    _match_strategy: MatchStrategy,
    sort_configs: Vec<SortConfig>,
    limit: Option<usize>,
    output_format: OutputFormat,
    delimiter: Option<String>,
    pretty: bool,
    template: Option<String>,
) -> Result<()> {
    let db_path = zet::core::collection_db_file(root);
    let db = DB::open(db_path)?;

    let separator = delimiter.unwrap_or("\n".into());

    // Build query from CLI args
    let mut query = DocumentQuery::new();

    if !ids.is_empty() {
        query = query.with_ids(ids);
    }
    if !titles.is_empty() {
        query = query.with_titles(titles);
    }
    if !paths.is_empty() {
        query = query.with_paths(paths);
    }
    if !tags.is_empty() {
        query = query.with_tags(tags);
    }
    if tagless {
        query = query.tagless();
    }
    if !exclude_list.is_empty() {
        query = query.exclude_ids(exclude_list);
    }
    if !exclude_by_path.is_empty() {
        query = query.exclude_paths(exclude_by_path);
    }
    if let Some(ts) = created {
        query = query.created(ts);
    }
    if let Some(ts) = modified {
        query = query.modified(ts);
    }
    if let Some(ts) = created_before {
        query = query.created_before(ts);
    }
    if let Some(ts) = created_after {
        query = query.created_after(ts);
    }
    if let Some(ts) = modified_before {
        query = query.modified_before(ts);
    }
    if let Some(ts) = modified_after {
        query = query.modified_after(ts);
    }
    if !links_to.is_empty() {
        query = query.links_to(links_to);
    }
    if !links_from.is_empty() {
        query = query.links_from(links_from);
    }

    // Add sorting
    for SortConfig { by, order } in sort_configs {
        let query_by = match by {
            SortByOption::Modified => QuerySortByOption::Modified,
            SortByOption::Created => QuerySortByOption::Created,
            SortByOption::Id => QuerySortByOption::Id,
            SortByOption::Path => QuerySortByOption::Path,
            SortByOption::Title => QuerySortByOption::Title,
        };
        let query_order = match order {
            SortOrder::Ascending => QuerySortOrder::Ascending,
            SortOrder::Descending => QuerySortOrder::Descending,
        };
        query = query.order_by(query_by, query_order);
    }

    // Add limit
    if let Some(n) = limit {
        query = query.limit(n);
    }

    let documents = query.execute(&db)?;

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
