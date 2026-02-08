mod helpers;

// use assert_cmd::prelude::*;
use helpers::{cli::*, db::*, *};

#[test]
fn test_init_creates_workspace() {
    let (temp, workspace) = setup_temp_workspace();

    // Run init command
    run_cli_cmd(&["init"], &workspace).assert().success();

    // Verify .zet directory exists
    let zet_directory = zet::core::collection_config_dir(&workspace);
    assert!(zet_directory.exists(), ".zet directory should exist");
    assert!(zet_directory.is_dir(), ".zet should be a directory");

    // Verify database exists
    let database_path = zet::core::collection_db_file(&workspace);
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
