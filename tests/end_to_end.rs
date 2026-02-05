mod helpers;

// use assert_cmd::prelude::*;
use helpers::{cli::*, db::*, *};
// use std::path::PathBuf;

// ============================================================================
// A. Init Tests
// ============================================================================

#[test]
fn test_init_creates_workspace() {
    let (temp, workspace) = setup_temp_workspace();

    // Run init command
    run_cli_cmd(&["init"], &workspace).assert().success();

    // Verify .zet directory exists
    let zet_directory = zet_dir(&workspace);
    assert!(zet_directory.exists(), ".zet directory should exist");
    assert!(zet_directory.is_dir(), ".zet should be a directory");

    // Verify database exists
    let database_path = db_path(&workspace);
    assert!(database_path.exists(), "db.sqlite should exist");

    // Verify database has correct schema
    let db = open_test_db(&workspace);

    // Check that tables exist by attempting to count rows
    assert_eq!(count_documents(&db), 0);
    assert_eq!(count_tags(&db), 0);
    assert_eq!(count_links(&db), 0);
    assert_eq!(count_headings(&db), 0);
    assert_eq!(count_tasks(&db), 0);
}

#[test]
fn test_init_fails_without_force() {
    let (temp, workspace) = setup_temp_workspace();

    // First init should succeed
    run_cli_cmd(&["init"], &workspace).assert().success();

    // Second init should fail without --force
    run_cli_cmd(&["init"], &workspace).assert().failure();

    // Third init with --force should succeed
    run_cli_cmd(&["init", "--force"], &workspace)
        .assert()
        .success();
}

// ============================================================================
// B. Index Tests
// ============================================================================

#[test]
fn test_index_new_documents() {
    let (temp, workspace) = setup_temp_workspace();
    copy_fixture_to_temp("knowledge-base", &temp).unwrap();

    run_cli_cmd(&["init"], &workspace).assert().success();
    run_cli_cmd(&["index"], &workspace).assert().success();

    let db = open_test_db(&workspace);
    let doc_count = count_documents(&db);

    assert_eq!(doc_count, 8);
}

#[test]
fn test_document_ids_match_disk_slugification() {
    let (temp, workspace) = setup_temp_workspace();
    copy_fixture_to_temp("knowledge-base", &temp).unwrap();

    // Initialize and index the workspace
    run_cli_cmd(&["init"], &workspace).assert().success();
    run_cli_cmd(&["index"], &workspace).assert().success();

    // Get document IDs from database with frontmatter info
    let db = open_test_db(&workspace);
    let db_docs = get_document_ids_with_frontmatter_info(&db);

    // Get document IDs by slugifying paths from disk
    let disk_paths = zet::core::workspace_paths(&workspace).expect("Failed to get workspace paths");
    let disk_ids: Vec<_> = disk_paths
        .iter()
        .map(|path| zet::core::path_to_id(&workspace, path))
        .collect();

    // We should have the same number of documents in DB as on disk
    assert_eq!(
        db_docs.len(),
        disk_ids.len(),
        "Database should have same number of documents as disk"
    );

    // For documents WITHOUT custom ID in frontmatter, their DB ID should match the path-based ID
    // For documents WITH custom ID in frontmatter, their DB ID should NOT be in disk_ids
    for (db_id, has_custom_id) in db_docs {
        if has_custom_id {
            assert!(
                !disk_ids.contains(&db_id),
                "Document {} has custom ID in frontmatter, so should NOT match path-based ID",
                db_id.0
            );
        } else {
            assert!(
                disk_ids.contains(&db_id),
                "Document {} has no custom ID in frontmatter, so should match path-based ID",
                db_id.0
            );
        }
    }
}

#[test]
/// We check that when indexing, if the 'id' and 'title' fields occur in the frontmatter
/// they are used for the id and title field in the db.
fn test_frontmatter_title_and_id_override() {
    let (temp, workspace) = setup_temp_workspace();
    copy_fixture_to_temp("knowledge-base", &temp).unwrap();

    // Initialize and index the workspace
    run_cli_cmd(&["init"], &workspace).assert().success();
    run_cli_cmd(&["index"], &workspace).assert().success();

    let db = open_test_db(&workspace);

    // Test document with custom ID and title in frontmatter
    let custom_doc = get_document_by_id(&db, "my-custom-document-id");
    assert!(
        custom_doc.is_some(),
        "Document with custom ID 'my-custom-document-id' should exist"
    );
    let (id, title) = custom_doc.unwrap();
    assert_eq!(
        id, "my-custom-document-id",
        "ID should match frontmatter id field"
    );
    assert_eq!(
        title, "Custom Title From Frontmatter",
        "Title should match frontmatter title field, not the H1 heading"
    );

    // Test document with only custom title (ID should be generated from path)
    let expected_id = zet::core::path_to_id(&workspace, &workspace.join("custom-title-only.md"));
    let title_only_doc = get_document_by_id(&db, &expected_id.0);
    assert!(
        title_only_doc.is_some(),
        "Document with path-generated ID should exist"
    );
    let (id, title) = title_only_doc.unwrap();
    assert_eq!(id, expected_id.0, "ID should be generated from path");
    assert_eq!(
        title, "Title Only From Frontmatter",
        "Title should match frontmatter title field"
    );
}
