use pulldown_cmark::Parser;
use rusqlite::{params, params_from_iter};
use serde_json::{Value, json};
use sql_minifier::macros::minify_sql as sql;
use std::ops::Range;
use zet::core::db::{DbDelete, DbUpdate};
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
    let updated_ids = updated.iter().map(|(id, _, _, _, _)| id.clone()).collect();

    Document::delete(&mut db, removed)?;
    DocumentLink::delete(&mut db, updated_ids)?;

    // parse and collect the data to be inserted into the db
    let mut documents = Vec::with_capacity(new.len() + updated.len());
    let mut links = Vec::new();
    process_new_documents(&config, new, &mut documents, &mut links)?;
    process_existing_documents(&config, updated, &mut documents, &mut links)?;

    // and then we update the db
    let doc_ids = Document::update(&mut db, documents)?;

    Ok(())
}

fn process_new_documents(
    config: &Config,
    new: Vec<DocumentPath>,
    documents: &mut Vec<Document>,
    links: &mut Vec<UnresolovedLink>,
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

fn extract_links_from_ast(
    links: &mut Vec<UnresolovedLink>,
    document_id: &DocumentId,
    nodes: &Vec<Node>,
) {
    for node in nodes {
        match node {
            // links
            Node::WikiLink { target, .. } => links.push(UnresolovedLink {
                from: document_id.clone(),
                to: target.clone(),
            }),
            Node::InlineLink { target, .. } => links.push(UnresolovedLink {
                from: document_id.clone(),
                to: target.clone(),
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
    links: &mut Vec<UnresolovedLink>,
) -> Result<()> {
    for (id, path, modified, created, hash) in updated {
        let content = std::fs::read_to_string(&path.0)?;

        let (frontmatter, document) = zet::core::parser::parse(
            FrontMatterParser::new(config.front_matter_format),
            zet::core::parser::DocumentParser::new(),
            content,
        )?;
        let frontmatter = frontmatter.unwrap_or(Value::Null);
        let title = extract_title_from_frontmatter(&frontmatter)
            .or_else(|| extract_title_from_ast(&document))
            .unwrap_or("".into());

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

struct DocumentData {
    document: Document,
}

struct UnresolovedLink {
    from: DocumentId,
    /// unresolved link target, might or might not map to a document_id
    to: String,
}

fn delete_document_links(db: &mut DB, document_ids: &Vec<&str>) -> Result<()> {
    let tx = db.transaction()?;
    {
        let mut query = tx.prepare(sql!("delete from document_link where from_id = ?"))?;

        for id in document_ids {
            query.execute([id])?;
        }
    }
    tx.commit()?;
    Ok(())
}
