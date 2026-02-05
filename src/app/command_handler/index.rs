use pulldown_cmark::Parser;
use rusqlite::{params, params_from_iter};
use serde_json::{Value, json};
use sql_minifier::macros::minify_sql as sql;
use std::ops::Range;
use zet::core::db::{DbDelete, DbInsert, DbUpdate};
use zet::core::parser::DocumentParserOptions;
use zet::core::parser::ast_nodes::{Node, TaskListMarker};
use zet::core::path_to_id;
use zet::core::types::heading::NewDocumentHeading;
use zet::core::types::link::{DocumentLink, DocumentLinkSource, NewDocumentLink};
use zet::core::types::task::{DocumentTask, NewDocumentTask};
use zet::core::types::{RangeEnd, RangeStart};
use zet::core::{extract_id_from_frontmatter, extract_title_from_ast, extract_title_from_frontmatter};
use zet::preamble::*;
use zet::{
    config::Config,
    core::{
        db::DB,
        parser::{
            FrontMatterParser,
            ast_nodes::{self, NodeKind},
        },
        types::document::{
            CreatedTimestamp, Document, DocumentId, DocumentPath, ModifiedTimestamp,
        },
    },
};

pub fn handle_command(config: Config, _force: bool) -> Result<()> {
    let root = &config.root;
    let db_path = zet::core::db_dir(root);
    let mut db = DB::open(db_path)?;

    // we figure out which documents we need to process,reprocess and delete
    let (new, updated, removed) = zet::core::collection_status(root, &db);

    log::info!(
        "collection status since last index: n_new={}, n_updated={}, n_removed={}",
        new.len(),
        updated.len(),
        removed.len()
    );

    // Delete removed documents. Associated data (links, headings) will be
    //
    // removed as well by trigger
    Document::delete(&mut db, &removed)?;

    // parse and collect the data to be inserted into the db
    let mut documents = Vec::with_capacity(new.len() + updated.len());
    let mut links = Vec::new();
    let mut headings = Vec::new();
    let mut tasks = Vec::new();
    // let mut headings = Vec::new();
    process_new_documents(
        &config,
        new,
        &mut documents,
        &mut links,
        &mut headings,
        &mut tasks,
    )?;
    process_existing_documents(
        &config,
        updated,
        &mut documents,
        &mut links,
        &mut headings,
        &mut tasks,
    )?;

    // Perform an upsert on the documents. This will clear any associated data
    // as well
    Document::update(&mut db, &documents)?;

    // links needs to be handled in a special. We want to resolve the link
    // target to some actual document
    let resolved_links = resolve_links(&db, links)?;
    DocumentLink::insert(&mut db, &resolved_links)?;
    DocumentTask::insert(&mut db, &tasks)?;

    Ok(())
}

fn resolve_links(db: &DB, unresolved_links: Vec<UnresolvedLink>) -> Result<Vec<NewDocumentLink>> {
    let mut links = Vec::new();

    // linear search for now!
    let ids: Vec<DocumentId> = db
        .prepare(sql!("select id from document"))?
        .query_map([], |r| r.get(0))?
        .map(|f| f.map_err(From::from))
        .collect::<Result<Vec<DocumentId>>>()?;

    for link in unresolved_links {
        let res = ids
            .iter()
            .find(|id| link.to.ends_with(&id.0))
            .map(|v| v.to_owned());
        links.push(NewDocumentLink {
            from: link.from.into(),
            to: res.map(From::from),
            range_start: link.range_start,
            range_end: link.range_end,
        })
    }

    Ok(links)
}

fn process_new_documents(
    config: &Config,
    new: Vec<DocumentPath>,
    documents: &mut Vec<Document>,
    links: &mut Vec<UnresolvedLink>,
    headings: &mut Vec<NewDocumentHeading>,
    tasks: &mut Vec<NewDocumentTask>,
) -> Result<()> {
    for DocumentPath(path) in new {
        // metadata
        let metadata = std::fs::metadata(&path)?;
        let modified = ModifiedTimestamp(metadata.modified().map(TryFrom::try_from)??);
        let created = CreatedTimestamp(metadata.created().map(TryFrom::try_from)??);

        let content = std::fs::read_to_string(&path)?;
        // hash
        let hash = zet::core::hash(&content);

        // frontmatter and ast
        let (frontmatter, document) = zet::core::parser::parse(
            FrontMatterParser::new(config.front_matter_format),
            zet::core::parser::DocumentParser::new(),
            content,
        )?;
        let frontmatter = frontmatter.unwrap_or(serde_json::Value::Null);

        // id - check frontmatter first, then fall back to path-based generation
        let id = extract_id_from_frontmatter(&frontmatter)
            .unwrap_or_else(|| path_to_id(&config.root, &path));

        // title
        let title = extract_title_from_frontmatter(&frontmatter)
            .or_else(|| extract_title_from_ast(&document))
            .unwrap_or("".into());

        // links
        extract_links_from_ast(links, &id, &document);
        extract_headings_from_ast(headings, &id, &document);
        extract_tasks_from_ast(tasks, &id, &document);

        // documents
        documents.push(Document {
            id,
            title,
            path: DocumentPath(path),
            hash,
            modified,
            created,
            data: frontmatter,
        });
    }

    Ok(())
}

fn process_existing_documents(
    config: &Config,
    updated: Vec<(
        zet::core::types::document::DocumentId,
        DocumentPath,
        zet::core::types::document::ModifiedTimestamp,
        zet::core::types::document::CreatedTimestamp,
        u32,
    )>,

    documents: &mut Vec<Document>,
    links: &mut Vec<UnresolvedLink>,
    headings: &mut Vec<NewDocumentHeading>,
    tasks: &mut Vec<NewDocumentTask>,
) -> Result<()> {
    for (id, path, modified, created, hash) in updated {
        let content = std::fs::read_to_string(&path.0)?;

        // frontmatter and ast
        let (frontmatter, document) = zet::core::parser::parse(
            FrontMatterParser::new(config.front_matter_format),
            zet::core::parser::DocumentParser::new(),
            content,
        )?;
        // frontmatter and ast
        let frontmatter = frontmatter.unwrap_or(Value::Null);
        // title
        let title = extract_title_from_frontmatter(&frontmatter)
            .or_else(|| extract_title_from_ast(&document))
            .unwrap_or("".into());

        // links
        extract_links_from_ast(links, &id, &document);
        extract_headings_from_ast(headings, &id, &document);
        extract_tasks_from_ast(tasks, &id, &document);

        documents.push(Document {
            id,
            title,
            path,
            hash,
            modified,
            created,
            data: frontmatter,
        });
    }

    Ok(())
}

struct UnresolvedLink {
    range_start: RangeStart,
    range_end: RangeEnd,
    from: DocumentLinkSource,
    /// unresolved link target, might or might not map to a document_id
    to: String,
}

fn extract_links_from_ast(
    links: &mut Vec<UnresolvedLink>,
    document_id: &DocumentId,
    nodes: &Vec<Node>,
) {
    for node in nodes {
        match node {
            // links
            Node::WikiLink { target, range, .. } => links.push(UnresolvedLink {
                from: document_id.clone().into(),
                to: target.clone(),
                range_start: range.start,
                range_end: range.end,
            }),
            Node::InlineLink { target, range, .. } => links.push(UnresolvedLink {
                from: document_id.clone().into(),
                to: target.clone(),
                range_start: range.start,
                range_end: range.end,
            }),
            // container nodes
            Node::Heading { children, .. } => extract_links_from_ast(links, document_id, children),
            Node::Paragraph { children, .. } => {
                extract_links_from_ast(links, document_id, children)
            }
            Node::BlockQuote { children, .. } => {
                extract_links_from_ast(links, document_id, children)
            }
            Node::List { children, .. } => extract_links_from_ast(links, document_id, children),
            Node::Item { children, .. } => extract_links_from_ast(links, document_id, children),
            Node::CodeBlock { children, .. } => {
                extract_links_from_ast(links, document_id, children)
            }
            // ignore the rest
            _ => {}
        }
    }
}

fn extract_headings_from_ast(
    headings: &mut Vec<NewDocumentHeading>,
    document_id: &DocumentId,
    nodes: &Vec<Node>,
) {
    for node in nodes {
        match node {
            Node::Heading {
                range,
                id,
                classes,
                attributes,
                level,
                content,
                children,
            } => {
                let metadata = json!({
                    "id": id,
                    "classes": classes,
                    "attributes": attributes
                });
                let range_start = range.start;
                let range_end = range.end;
                headings.push(NewDocumentHeading {
                    document_id: document_id.clone(),
                    content: content.to_owned(),
                    level: *level,
                    metadata,
                    range_start,
                    range_end,
                });
                extract_headings_from_ast(headings, document_id, children);
            }
            // no other node should contain heading nodes
            _ => {}
        }
    }
}

// TODO this should probably be extended to capture that tasks typically have subtasks
fn extract_tasks_from_ast(
    tasks: &mut Vec<NewDocumentTask>,
    document_id: &DocumentId,
    nodes: &Vec<Node>,
) {
    for node in nodes {
        match node {
            Node::Heading { children, .. } => extract_tasks_from_ast(tasks, document_id, children),
            Node::List { children, .. } => extract_tasks_from_ast(tasks, document_id, children),
            Node::Item {
                range,
                task_list_marker,
                children: _,
                sub_lists,
            } => {
                match task_list_marker {
                    TaskListMarker::UnChecked | TaskListMarker::Checked => {
                        let checked = match task_list_marker {
                            TaskListMarker::UnChecked => false,
                            TaskListMarker::Checked => true,
                            _ => unreachable!(),
                        };
                        let content = String::new();

                        tasks.push(NewDocumentTask {
                            document_id: document_id.to_owned(),
                            parent_id: None,
                            checked,
                            content,
                            range_start: range.start,
                            range_end: range.end,
                        });
                    }
                    TaskListMarker::NoCheckmark => {}
                }
                extract_tasks_from_ast(tasks, document_id, sub_lists);
            }
            _ => {}
        }
    }
}
