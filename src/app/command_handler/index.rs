use color_eyre::eyre::eyre;
use serde_json::json;
use sql_minifier::macros::minify_sql as sql;
use twox_hash::XxHash3_64;
use zet::{
    config::Config,
    core::{
        db::DB,
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
    log::debug!("{:?}", new);

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
        let hash = XxHash3_64::oneshot(content.as_bytes());

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
        u64,
    )>,
) -> Result<Vec<DocumentData>> {
    todo!();
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
