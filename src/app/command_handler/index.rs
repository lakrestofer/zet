use std::ops::Range;

use color_eyre::eyre::eyre;
use rusqlite::{params, params_from_iter};
use serde_json::json;
use sql_minifier::macros::minify_sql as sql;
use zet::preamble::*;
use zet::{
    config::Config,
    core::{
        db::{DB, DbCrud},
        hasher::hash,
        parser::{
            FrontMatterParser,
            ast_nodes::{self, NodeKind, Ranged},
        },
        types::{
            CreatedTimestamp, Document, DocumentId, DocumentPath, JsonData, ModifiedTimestamp, Node,
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
        db_nodes.push((doc.document.id.clone(), doc.content));
        db_documents.push(doc.document);
    }

    Document::upsert(db, db_documents)?;

    for (id, nodes) in db_nodes {
        db_insert_nodes(db, id, nodes)?;
    }

    Ok(())
}

fn build_db_nodes(
    parent: Option<usize>,
    nodes: Vec<ast_nodes::Node>,
    db_nodes: &mut Vec<(NodeKind, Range<usize>, Option<usize>, JsonData)>,
) {
    for node in nodes {
        use ast_nodes::{Node::*, *};
        let kind = node.kind();
        let range = node.range().to_owned();

        match node {
            Document(d) => {
                db_nodes.push((kind, range, parent, Default::default()));
                build_db_nodes(Some(db_nodes.len() - 1), d.children, db_nodes);
            }
            Paragraph(p) => {
                db_nodes.push((kind, range, parent, Default::default()));
                build_db_nodes(Some(db_nodes.len() - 1), p.children, db_nodes);
            }
            BlockQuote(q) => {
                db_nodes.push((kind, range, parent, Default::default()));
                build_db_nodes(Some(db_nodes.len() - 1), q.children, db_nodes);
            }
            Heading(h) => {
                let iter = h
                    .attributes
                    .into_iter()
                    .map(|(k, v)| (k, serde_json::to_value(v).unwrap()));
                let map = serde_json::Map::from_iter(iter);
                let data = json!({
                   "id": h.id,
                   "classes": h.classes,
                   "attributes": map,
                   "level": h.level
                });
                db_nodes.push((kind, range, parent, JsonData(data)));
                build_db_nodes(Some(db_nodes.len() - 1), h.children, db_nodes);
            }
            Text(_) => {
                db_nodes.push((kind, range, parent, Default::default()));
            }
            TextDecoration(td) => {
                let data = json!({
                    "kind": td.kind,
                });
                db_nodes.push((kind, range, parent, JsonData(data)));
                build_db_nodes(Some(db_nodes.len() - 1), td.children, db_nodes);
            }
            Html(html) => {
                let data = json!({
                    "text": html.text,
                });
                db_nodes.push((kind, range, parent, JsonData(data)));
            }
            FootnoteReference(fr) => {
                let data = json!({
                    "name": fr.name,
                });
                db_nodes.push((kind, range, parent, JsonData(data)));
            }
            FootnoteDefinition(fd) => {
                let data = json!({
                    "name": fd.name,
                });
                db_nodes.push((kind, range, parent, JsonData(data)));
                build_db_nodes(Some(db_nodes.len() - 1), fd.children, db_nodes);
            }
            InlineLink(inline_link) => todo!(),
            ReferenceLink(reference_link) => todo!(),
            ShortcutLink(shortcut_link) => todo!(),
            AutoLink(auto_link) => todo!(),
            WikiLink(wiki_link) => todo!(),
            LinkReference(link_reference) => todo!(),
            InlineImage(inline_image) => todo!(),
            ReferenceImage(reference_image) => todo!(),
            List(list) => todo!(),
            Item(item) => todo!(),
            TaskListMarker(task_list_marker) => todo!(),
            SoftBreak(soft_break) => todo!(),
            HardBreak(hard_break) => todo!(),
            Code(code) => todo!(),
            CodeBlock(code_block) => todo!(),
            HorizontalRule(horizontal_rule) => todo!(),
            Table(table) => todo!(),
            TableHead(table_head) => todo!(),
            TableRow(table_row) => todo!(),
            TableCell(table_cell) => todo!(),
            MetadataBlock(metadata_block) => todo!(),
            DisplayMath(display_math) => todo!(),
            InlineMath(inline_math) => todo!(),
            // NotImplemented(_) => todo!(),
            _ => {}
        }

        // let data = serde_json::to_value(n).unwrap_or_else(|e| {
        //     log::error!("could not convert node to json representation: {:?}", e);
        //     serde_json::Value::Null
        // });
    }
}

fn db_insert_nodes(
    db: &mut DB,
    document_id: DocumentId,
    nodes: Vec<ast_nodes::Node>,
) -> Result<()> {
    let mut db_nodes = Vec::new();

    build_db_nodes(None, nodes, &mut db_nodes);

    // let tx = db.transaction()?;
    // {
    //     let mut query = tx.prepare(sql!(
    //         r#"
    //         insert into
    //             node
    //         values (
    //             ?1,
    //             ?2,
    //             ?3,
    //             ?4,
    //             ?5,
    //             ?6,
    //         ) returning id
    //     "#
    //     ))?;
    //     let mut ids = Vec::new();
    //     for (document_id, node_kind, range_start, range_end, json_data) in db_nodes {
    //         let id: i64 = query.query_row(
    //             params![document_id, node_kind, range_start, range_end, json_data],
    //             |r| r.get(0),
    //         )?;
    //         ids.push(id);
    //     }
    // }
    // tx.commit()?;

    Ok(())
}

fn process_new_documents(config: &Config, new: Vec<DocumentPath>) -> Result<Vec<DocumentData>> {
    let mut document_data = Vec::new();

    for DocumentPath(path) in new {
        let id = path_to_id(path.clone());

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

fn path_to_id(mut path: PathBuf) -> DocumentId {
    path.set_extension("");
    DocumentId(
        path.to_str()
            .expect("document path did not constitute valid utf8")
            .to_owned(),
    )
}

struct PartialNode {
    pub document_id: DocumentId,
    pub parent_id: Option<i64>,
    pub node_type: NodeKind,
    pub range_start: usize,
    pub range_end: usize,
    pub data: JsonData,
}
