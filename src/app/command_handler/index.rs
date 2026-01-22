use pulldown_cmark::Parser;
use rusqlite::{params, params_from_iter};
use serde_json::{Value, json};
use sql_minifier::macros::minify_sql as sql;
use std::ops::Range;
use zet::core::db::{DbDelete, DbInsert, DbUpdate};
use zet::core::parser::DocumentParserOptions;
use zet::core::parser::ast_nodes::Node;
use zet::core::path_to_id;
use zet::core::types::link::DocumentLink;
use zet::core::{extract_title_from_ast, extract_title_from_frontmatter};
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

    // delete stale data
    let updated_ids: Vec<_> = updated.iter().map(|(id, _, _, _, _)| id.clone()).collect();

    Document::delete(&mut db, &removed)?;
    DocumentLink::delete(&mut db, &updated_ids)?;

    // parse and collect the data to be inserted into the db
    let mut documents = Vec::with_capacity(new.len() + updated.len());
    let mut links = Vec::new();
    // let mut headings = Vec::new();
    process_new_documents(&config, new, &mut documents, &mut links)?;
    process_existing_documents(&config, updated, &mut documents, &mut links)?;

    //  perform an upsert
    Document::update(&mut db, &documents)?;

    // links needs to be handled in a special. We want to resolve the link
    // target to some actual document
    let resolved_links = resolve_links(&db, links)?;
    DocumentLink::insert(&mut db, &resolved_links)?;

    Ok(())
}

fn resolve_links(db: &DB, unresolved_links: Vec<UnresolvedLink>) -> Result<Vec<DocumentLink>> {
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
        links.push(DocumentLink {
            from: link.from,
            to: res,
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
) -> Result<()> {
    for DocumentPath(path) in new {
        let id = path_to_id(&config.root, &path);

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

        // title
        let title = extract_title_from_frontmatter(&frontmatter)
            .or_else(|| extract_title_from_ast(&document))
            .unwrap_or("".into());

        // links
        extract_links_from_ast(links, &id, &document);

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
    range_start: usize,
    range_end: usize,
    from: DocumentId,
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
                from: document_id.clone(),
                to: target.clone(),
                range_start: range.start,
                range_end: range.end,
            }),
            Node::InlineLink { target, range, .. } => links.push(UnresolvedLink {
                from: document_id.clone(),
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
