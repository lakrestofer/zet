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
            CreatedTimestamp, Document, DocumentId, DocumentPath, JsonData, ModifiedTimestamp,
            Node, NodeKind,
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
        db_insert_nodes(db, id, None, nodes)?;
    }

    Ok(())
}

fn db_insert_nodes(
    db: &mut DB,
    document_id: DocumentId,
    parent_id: Option<i64>,
    nodes: Vec<ast_nodes::Node>,
) -> Result<()> {
    for node in nodes {
        use ast_nodes::Node as AstNode;
        let node: PartialNode = match node {
            AstNode::Heading(heading) => todo!(),
            AstNode::Paragraph(paragraph) => todo!(),
            AstNode::BlockQuote(block_quote) => todo!(),
            AstNode::Text(text) => todo!(),
            AstNode::TextDecoration(text_decoration) => todo!(),
            AstNode::Html(html) => todo!(),
            AstNode::FootnoteReference(footnote_reference) => todo!(),
            AstNode::FootnoteDefinition(footnote_definition) => todo!(),
            AstNode::DefinitionList(definition_list) => todo!(),
            AstNode::DefinitionListTitle(definition_list_title) => todo!(),
            AstNode::DefinitionListDefinition(definition_list_definition) => todo!(),
            AstNode::InlineLink(inline_link) => todo!(),
            AstNode::ReferenceLink(reference_link) => todo!(),
            AstNode::ShortcutLink(shortcut_link) => todo!(),
            AstNode::AutoLink(auto_link) => todo!(),
            AstNode::WikiLink(wiki_link) => todo!(),
            AstNode::LinkReference(link_reference) => todo!(),
            AstNode::InlineImage(inline_image) => todo!(),
            AstNode::ReferenceImage(reference_image) => todo!(),
            AstNode::List(list) => todo!(),
            AstNode::Item(item) => todo!(),
            AstNode::TaskListMarker(task_list_marker) => todo!(),
            AstNode::SoftBreak(soft_break) => todo!(),
            AstNode::HardBreak(hard_break) => todo!(),
            AstNode::Code(code) => todo!(),
            AstNode::CodeBlock(code_block) => todo!(),
            AstNode::HorizontalRule(horizontal_rule) => todo!(),
            AstNode::Table(table) => todo!(),
            AstNode::TableHead(table_head) => todo!(),
            AstNode::TableRow(table_row) => todo!(),
            AstNode::TableCell(table_cell) => todo!(),
            AstNode::MetadataBlock(metadata_block) => todo!(),
            AstNode::DisplayMath(display_math) => todo!(),
            AstNode::InlineMath(inline_math) => todo!(),
            _ => return Err(eyre!("cannot convert ast node to db node")),
        };
    }

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
