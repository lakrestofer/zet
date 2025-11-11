use sql_minifier::macros::minify_sql as sql;
use zet::{
    config::Config,
    core::{collection::CollectionDocumentStatus, db::DB},
};

use crate::app::preamble::*;

pub fn handle_command(config: Config) -> Result<()> {
    let root = &config.root;
    let db_path = zet::core::paths::db_dir(root);
    let mut db = DB::open(db_path)?;

    let CollectionDocumentStatus {
        new,
        updated,
        removed,
    } = zet::core::collection::collection_status(root, &mut db)?;
    log::debug!("{:?}", new);

    // we remove the old documents from the db
    let removed_ids = removed.iter().map(|r| r.0.as_str()).collect();
    delete_documents(&mut db, removed_ids)?;

    // then we remove the cached description of the document contents
    // (since it is now stale)
    let updated_ids = updated.iter().map(|u| u.0.as_str()).collect();
    delete_nodes(&mut db, updated_ids)?;

    // we then

    Ok(())
}

fn delete_documents(db: &mut DB, document_ids: Vec<&str>) -> Result<()> {
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

fn delete_nodes(db: &mut DB, document_ids: Vec<&str>) -> Result<()> {
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
