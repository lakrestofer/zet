use std::ops::Range;
use std::path::Path;

use rusqlite::params;
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

pub fn handle_command(config: Config, force: bool) -> Result<()> {
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

    log::debug!("inserting document data: {:?}", db_nodes);
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
            InlineLink(il) => {
                let data = json!({
                    "url": il.url,
                    "title": il.title,
                });
                db_nodes.push((kind, range, parent, JsonData(data)));
                build_db_nodes(Some(db_nodes.len() - 1), il.children, db_nodes);
            }
            ReferenceLink(rl) => {
                let data = json!({
                    "reference": rl.reference,
                });
                db_nodes.push((kind, range, parent, JsonData(data)));
                build_db_nodes(Some(db_nodes.len() - 1), rl.children, db_nodes);
            }
            ShortcutLink(sl) => {
                db_nodes.push((kind, range, parent, Default::default()));
                build_db_nodes(Some(db_nodes.len() - 1), sl.children, db_nodes);
            }
            AutoLink(al) => {
                db_nodes.push((kind, range, parent, Default::default()));
                build_db_nodes(Some(db_nodes.len() - 1), al.children, db_nodes);
            }
            WikiLink(wl) => {
                db_nodes.push((kind, range, parent, Default::default()));
                build_db_nodes(Some(db_nodes.len() - 1), wl.children, db_nodes);
            }
            LinkReference(lr) => {
                let data = json!({
                    "name": lr.name,
                    "link": lr.link,
                    "title": lr.title,
                });
                db_nodes.push((kind, range, parent, JsonData(data)));
            }
            InlineImage(_) => {
                db_nodes.push((kind, range, parent, Default::default()));
            }
            ReferenceImage(ri) => {
                db_nodes.push((kind, range, parent, Default::default()));
            }
            List(l) => {
                db_nodes.push((kind, range, parent, Default::default()));
                build_db_nodes(Some(db_nodes.len() - 1), l.children, db_nodes);
            }
            Item(i) => {
                db_nodes.push((kind, range, parent, Default::default()));
                let id = db_nodes.len() - 1;
                build_db_nodes(Some(id), i.children, db_nodes);
                build_db_nodes(Some(id), i.sub_lists, db_nodes);
            }
            TaskListMarker(tlm) => {
                let data = json!({"checked": tlm.is_checked});
                db_nodes.push((kind, range, parent, JsonData(data)));
                // let id = db_nodes.len() - 1;
                // build_db_nodes(Some(id), ltm., db_nodes);
            }
            Code(code) => {
                let data = json!({"code": code.code});
                db_nodes.push((kind, range, parent, JsonData(data)));
            }
            CodeBlock(cb) => {
                let data = json!({
                    "tag": cb.tag,
                    "is_fenced": cb.is_fenced,
                });
                db_nodes.push((kind, range, parent, JsonData(data)));
                build_db_nodes(Some(db_nodes.len() - 1), cb.children, db_nodes);
            }
            HorizontalRule(_) => db_nodes.push((kind, range, parent, Default::default())),
            Table(table) => {
                let data = json!({
                    "header": table.header,
                    "rows": table.rows,
                });
                db_nodes.push((kind, range, parent, JsonData(data)));
            }
            DisplayMath(dm) => {
                let data = json!({
                    "text": dm.text,
                });
                db_nodes.push((kind, range, parent, JsonData(data)));
            }
            InlineMath(im) => {
                let data = json!({
                    "text": im.text,
                });
                db_nodes.push((kind, range, parent, JsonData(data)));
            }
            TableHead(_) | TableRow(_) | TableCell(_) => {
                panic!("should not be able to reach this!")
            }
            // NotImplemented(_) => todo!(),
            // SoftBreak(soft_break) => todo!(),
            // HardBreak(hard_break) => todo!(),
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
    // we build a list of all the nodes we are to insert, turning
    // the tree structure into a flat list
    // we then do the insertion in two steps
    // - insert all the nodes
    // - update any references to parent nodes

    let mut db_nodes = Vec::new();

    build_db_nodes(None, nodes, &mut db_nodes);

    let tx = db.transaction()?;
    {
        let mut query = tx
            .prepare(sql!(
                r#"
            insert into node (
                document_id,
                type,
                range_start,
                range_end,
                data
            ) values (
                ?1,
                ?2,
                ?3,
                ?4,
                jsonb(?5)
            ) returning id
        "#
            ))
            .unwrap();
        // first we insert all the nodes and gather their new ids
        let mut ids = Vec::new();
        for (node_kind, range, _, json_data) in db_nodes.iter() {
            let Range { start, end } = range;
            let id: i64 = query.query_row(
                params![document_id, node_kind, start, end, json_data],
                |r| r.get(0),
            )?;
            ids.push(id);
        }
        // then we check which nodes had a parent (that we then need to update)
        //
        // db_nodes had an incrementing parent_id that referred to the index of its parent
        // in the same list
        //
        // db_nodes = [_,_,_,3,3,3,3,_,_,_,_, 11, 11, 11]
        // ids =      [6,7,8,9,10,11,12,13,14,15,16,17,18, 19]
        // to_update = [(9,8), (10,8), (11,8), (12,8), (17, 16), (18, 16), (19, 16))]
        let to_update = db_nodes
            .into_iter()
            .enumerate()
            .filter(|(_, (_, _, parent, _))| parent.is_some())
            .map(|(i, (_, _, parent, _))| (i, parent.unwrap()))
            .map(|(node_index, parent_index)| (ids[node_index], ids[parent_index]));

        let mut query = tx
            .prepare(sql!(
                r#"
                update node
                set
                    parent_id = ?2
                where
                    id = ?1
                "#
            ))
            .unwrap();
        for (node, parent) in to_update {
            query.execute(params![node, parent])?;
        }
    }
    tx.commit()?;

    Ok(())
}

fn process_new_documents(config: &Config, new: Vec<DocumentPath>) -> Result<Vec<DocumentData>> {
    let mut document_data = Vec::new();

    for DocumentPath(path) in new {
        let id = path_to_id(&config.root, path.clone());

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

fn path_to_id(root: &Path, mut path: PathBuf) -> DocumentId {
    path.set_extension("");
    let path = path.strip_prefix(root).unwrap();
    DocumentId(
        path.to_str()
            .expect("document path did not constitute valid utf8")
            .to_owned(),
    )
}
