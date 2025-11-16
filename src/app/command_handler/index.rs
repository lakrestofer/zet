use color_eyre::eyre::eyre;
use serde_json::json;
use sql_minifier::macros::minify_sql as sql;
use zet::{
    config::Config,
    core::{
        db::{DB, DbCrud},
        hasher::hash,
        parser::{FrontMatterParser, ast_nodes},
        types::{
            CreatedTimestamp, Document, DocumentId, DocumentPath, InternalLink, JsonData,
            ModifiedTimestamp, Node,
        },
    },
};

use crate::app::preamble::*;

pub fn handle_command(config: Config) -> Result<()> {
    let root = &config.root;
    let db_path = zet::core::paths::db_dir(root);
    let mut db = DB::open(db_path)?;

    // we figure out which documents we need to process,reprocess and delete
    let (new, updated, removed) = zet::core::collection::collection_status(root, &mut db);

    log::info!(
        "collection status since last index: n_new={}, n_updated={}, n_removed={}",
        new.len(),
        updated.len(),
        removed.len()
    );

    let removed_ids = removed.iter().map(|r| r.0.as_str()).collect();
    let updated_ids = updated
        .iter()
        .map(|(id, _, _, _, _)| id.0.as_str())
        .collect();

    // we delete old data
    delete_documents(&mut db, &removed_ids)?;
    delete_nodes(&mut db, &updated_ids)?;

    // parse and collect the data to be inserted into the db
    let mut documents = Vec::with_capacity(new.len() + updated.len());
    documents.extend(process_new_documents(&config, new)?);
    documents.extend(process_existing_documents(&config, updated)?);

    // and then we update the db
    db_insert(&mut db, documents)?;

    Ok(())
}

fn db_insert(db: &mut DB, documents: Vec<DocumentData>) -> Result<()> {
    let mut db_nodes = Vec::with_capacity(documents.len());
    let mut db_documents = Vec::with_capacity(documents.len());

    for doc in documents {
        db_nodes.push(doc.content);
        db_documents.push(doc.document);
    }

    Document::upsert(db, db_documents)?;

    Ok(())
}

fn process_new_documents(config: &Config, new: Vec<DocumentPath>) -> Result<Vec<DocumentData>> {
    let mut document_data = Vec::new();

    for DocumentPath(path) in new {
        let id = DocumentId(zet::core::slug::slugify(
            path.to_str().ok_or(eyre!("path is not valid utf8"))?,
        ));

        let metadata = std::fs::metadata(&path)?;
        let modified = ModifiedTimestamp(metadata.modified().map(From::from)?);
        let created = CreatedTimestamp(metadata.created().map(From::from)?);

        let content = std::fs::read_to_string(&path)?;
        let hash = zet::core::hasher::hash(&content);

        let (frontmatter, nodes) = zet::core::parser::parse(
            FrontMatterParser::new(config.front_matter_format),
            zet::core::parser::DocumentParser::new(),
            content,
        )?;
        let frontmatter = frontmatter.unwrap_or(JsonData(json!("{}")));

        document_data.push(DocumentData {
            document: Document {
                id: id,
                path: DocumentPath(path),
                hash,
                modified,
                created,
                data: frontmatter,
            },
            content: nodes.children,
        })
    }

    Ok(document_data)
}
fn process_existing_documents(
    config: &Config,
    updated: Vec<(
        zet::core::types::DocumentId,
        DocumentPath,
        zet::core::types::ModifiedTimestamp,
        zet::core::types::CreatedTimestamp,
        u32,
    )>,
) -> Result<Vec<DocumentData>> {
    let mut document_data = Vec::new();

    for (id, path, modified, created, hash) in updated {
        let content = std::fs::read_to_string(&path.0)?;

        let (frontmatter, nodes) = zet::core::parser::parse(
            FrontMatterParser::new(config.front_matter_format),
            zet::core::parser::DocumentParser::new(),
            content,
        )?;
        let frontmatter = frontmatter.unwrap_or(JsonData(json!("{}")));

        document_data.push(DocumentData {
            document: Document {
                id: id,
                path: path,
                hash: hash,
                modified: modified,
                created: created,
                data: frontmatter,
            },
            content: nodes.children,
        })
    }

    Ok(document_data)
}

struct DocumentData {
    document: Document,
    content: Vec<ast_nodes::Node>,
}

fn delete_documents(db: &mut DB, document_ids: &Vec<&str>) -> Result<()> {
    let tx = db.transaction()?;
    {
        let mut query = tx.prepare(sql!("delete from document where id = ?"))?;
        for id in document_ids {
            query.execute([id])?;
        }
    }
    tx.commit()?;
    Ok(())
}

fn delete_nodes(db: &mut DB, document_ids: &Vec<&str>) -> Result<()> {
    let tx = db.transaction()?;
    {
        let mut query = tx.prepare(sql!("delete from node where document_id = ?"))?;
        for id in document_ids {
            query.execute([id])?;
        }
    }
    tx.commit()?;
    Ok(())
}
