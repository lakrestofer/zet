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

    assert_eq!(doc_count, 6);
}

#[test]
fn test_document_ids_match_disk_slugification() {
    let (temp, workspace) = setup_temp_workspace();
    copy_fixture_to_temp("knowledge-base", &temp).unwrap();

    // Initialize and index the workspace
    run_cli_cmd(&["init"], &workspace).assert().success();
    run_cli_cmd(&["index"], &workspace).assert().success();

    // Get document IDs from database
    let db = open_test_db(&workspace);
    let mut db_ids = get_all_document_ids(&db);
    db_ids.sort();

    // Get document IDs by slugifying paths from disk
    let disk_paths = zet::core::workspace_paths(&workspace)
        .expect("Failed to get workspace paths");
    let mut disk_ids: Vec<_> = disk_paths
        .iter()
        .map(|path| zet::core::path_to_id(&workspace, path))
        .collect();
    disk_ids.sort();

    // The two lists should be identical
    assert_eq!(
        db_ids, disk_ids,
        "Document IDs in database should match IDs generated from disk paths"
    );
}
