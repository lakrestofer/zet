use rusqlite::{params, params_from_iter};
use serde_json::{Value, json};
use sql_minifier::macros::minify_sql as sql;
use std::ops::Range;
use zet::core::db::DbUpdate;
use zet::core::path_to_id;
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

    Document::update(db, db_documents)?;

    for (id, nodes) in db_nodes {
        let node_ids = db_insert_nodes(db, id, &nodes)?;

        // TODO finish insertion of links into db
        // let links = node_ids.into_iter().zip(nodes.into_iter()).filter_map(|(node_id, node)| match node {
        //     ast_nodes::Node::InlineLink { range, title, target } => todo!(),
        //     ast_nodes::Node::WikiLink { range, children } => todo!(),
        //     _ => None
        // })
    }

    Ok(())
}

fn build_db_nodes<'a>(
    parent: Option<usize>,
    nodes: &'a Vec<ast_nodes::Node>,
    db_nodes: &mut Vec<(NodeKind, &'a Range<usize>, Option<usize>, serde_json::Value)>,
) {
    for node in nodes {
        use ast_nodes::Node::*;

        let kind = node.kind();

        // we have two kinds of nodes, the container nodes, and leaf nodes
        match node {
            // leaf nodes
            HardBreak { range, .. }
            | Text { range, .. }
            | TextDecoration { range, .. }
            | Html { range, .. }
            | InlineLink { range, .. }
            | ReferenceLink { range, .. }
            | ShortcutLink { range, .. }
            | AutoLink { range, .. }
            | LinkReference { range, .. }
            | InlineImage { range, .. }
            | ReferenceImage { range, .. }
            | Code { range, .. }
            | HorizontalRule { range, .. }
            | DisplayMath { range, .. }
            | InlineMath { range, .. }
            | FootnoteDefinition { range, .. }
            | FootnoteReference { range, .. } => {
                let data = node.inner_json_data();
                db_nodes.push((kind, range, parent, data.into()));
            }
            WikiLink {
                range,
                title,
                target,
            } => {
                db_nodes.push((
                    kind,
                    range,
                    parent,
                    json!({"title": title, "target": target}),
                ));
            }
            // container nodes. We have those with metadata and without
            Heading {
                range,
                id,
                classes,
                attributes,
                level,
                content,
                children,
            } => {
                db_nodes.push((
                    kind,
                    range,
                    parent,
                    json!({
                        "id": id,
                        "classes": classes,
                        "attributes": attributes,
                        "level": level,
                        "content": content
                    }),
                ));
                build_db_nodes(Some(db_nodes.len() - 1), children, db_nodes);
            }
            Paragraph { children, range } => {
                db_nodes.push((kind, range, parent, Default::default()));
                build_db_nodes(Some(db_nodes.len() - 1), children, db_nodes);
            }
            BlockQuote { children, range } => {
                db_nodes.push((kind, range, parent, Default::default()));
                build_db_nodes(Some(db_nodes.len() - 1), children, db_nodes);
            }
            List {
                start_index: _,
                children,
                range,
            } => {
                db_nodes.push((kind, range, parent, Default::default()));
                build_db_nodes(Some(db_nodes.len() - 1), children, db_nodes);
            }
            Item {
                children,
                // TODO include this in the metadata
                task_list_marker: _,
                sub_lists,
                range,
            } => {
                // TODO. Maybe include number of items in added metadata?
                db_nodes.push((kind, range, parent, Default::default()));
                let id = db_nodes.len() - 1;
                build_db_nodes(Some(id), children, db_nodes);
                build_db_nodes(Some(id), sub_lists, db_nodes);
            }
            CodeBlock {
                range,
                tag,
                is_fenced: _,
                children,
            } => {
                db_nodes.push((kind, range, parent, json!({"tag": tag})));
                let id = db_nodes.len() - 1;
                build_db_nodes(Some(id), children, db_nodes);
            }
            Table {
                range,
                header: _,
                column_alignment: _,
                rows: _,
            } => {
                db_nodes.push((kind, range, parent, Default::default()));
                // TODO how should we represent tables in the db such that we may query?
            }
        }
    }
}

fn db_insert_nodes(
    db: &mut DB,
    document_id: DocumentId,
    nodes: &Vec<ast_nodes::Node>,
) -> Result<Vec<i64>> {
    // we build a list of all the nodes we are to insert, turning
    // the tree structure into a flat list
    // we then do the insertion in two steps
    // - insert all the nodes
    // - update any references to parent nodes

    let mut db_nodes = Vec::new();
    let mut ids = Vec::new();

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
        for (node_kind, range, _, json_data) in db_nodes.iter() {
            let Range { start, end } = range;
            let id: i64 = query.query_row(
                params_from_iter([
                    serde_json::to_value(&document_id)?,
                    serde_json::to_value(node_kind)?,
                    serde_json::to_value(start)?,
                    serde_json::to_value(end)?,
                    serde_json::to_value(json_data)?,
                ]),
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

    Ok(ids)
}

fn process_new_documents(config: &Config, new: Vec<DocumentPath>) -> Result<Vec<DocumentData>> {
    let mut document_data = Vec::new();

    for DocumentPath(path) in new {
        let id = path_to_id(&config.root, &path);

        let metadata = std::fs::metadata(&path)?;
        let modified = ModifiedTimestamp(metadata.modified().map(TryFrom::try_from)??);
        let created = CreatedTimestamp(metadata.created().map(TryFrom::try_from)??);

        let content = std::fs::read_to_string(&path)?;
        let hash = zet::core::hash(&content);

        let (frontmatter, document) = zet::core::parser::parse(
            FrontMatterParser::new(config.front_matter_format),
            zet::core::parser::DocumentParser::new(),
            content,
        )?;
        let frontmatter = frontmatter.unwrap_or(serde_json::Value::Null);

        let title = extract_title_from_frontmatter(&frontmatter)
            .or_else(|| extract_title_from_ast(&document))
            .unwrap_or("".into());

        document_data.push(DocumentData {
            document: Document {
                id,
                title,
                path: DocumentPath(path),
                hash,
                modified,
                created,
                data: frontmatter,
            },
            content: document,
        })
    }

    Ok(document_data)
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
) -> Result<Vec<DocumentData>> {
    let mut document_data = Vec::new();

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

        document_data.push(DocumentData {
            document: Document {
                id,
                title,
                path,
                hash,
                modified,
                created,
                data: frontmatter,
            },
            content: document,
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
