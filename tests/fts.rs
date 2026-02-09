mod helpers;

use helpers::{cli::*, *};

/// Helper to setup FTS test workspace
fn setup_fts_workspace() -> (assert_fs::TempDir, std::path::PathBuf) {
    let (temp, workspace) = setup_temp_workspace();
    copy_fixture_to_temp("fts-test", &temp).unwrap();

    run_cli_cmd(&["init"], &workspace).assert().success();
    run_cli_cmd(&["index"], &workspace).assert().success();

    (temp, workspace)
}

// =============================================================================
// FTS (Full-Text Search) filters
// =============================================================================

#[test]
fn test_fts_search_body_content() {
    let (_temp, workspace) = setup_fts_workspace();

    // Search for "borrow checker" - should match rust-programming and memory-management
    let ids = query_document_ids(
        &workspace,
        &[
            "query",
            "--match",
            "borrow checker",
            "--output-format",
            "ids",
        ],
    );

    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&"rust-programming".to_string()));
    assert!(ids.contains(&"memory-management".to_string()));
}

#[test]
fn test_fts_search_unique_term() {
    let (_temp, workspace) = setup_fts_workspace();

    // Search for "cargo" - only in rust-programming
    let ids = query_document_ids(
        &workspace,
        &["query", "--match", "cargo", "--output-format", "ids"],
    );

    assert_eq!(ids.len(), 1);
    assert!(ids.contains(&"rust-programming".to_string()));
}

#[test]
fn test_fts_search_title() {
    let (_temp, workspace) = setup_fts_workspace();

    // Search for "tutorial" - matches python-tutorial title
    let ids = query_document_ids(
        &workspace,
        &["query", "--match", "tutorial", "--output-format", "ids"],
    );

    assert_eq!(ids.len(), 1);
    assert!(ids.contains(&"python-tutorial".to_string()));
}

#[test]
fn test_fts_search_no_results() {
    let (_temp, workspace) = setup_fts_workspace();

    // Search for term that doesn't exist
    let ids = query_document_ids(
        &workspace,
        &[
            "query",
            "--match",
            "nonexistentterm123",
            "--output-format",
            "ids",
        ],
    );

    assert_eq!(ids.len(), 0);
}

#[test]
fn test_fts_search_cooking_domain() {
    let (_temp, workspace) = setup_fts_workspace();

    // Search for "recipes" - only in cooking-recipes
    let ids = query_document_ids(
        &workspace,
        &["query", "--match", "recipes", "--output-format", "ids"],
    );

    assert_eq!(ids.len(), 1);
    assert!(ids.contains(&"cooking-recipes".to_string()));
}

#[test]
fn test_fts_search_with_tag_filter() {
    let (_temp, workspace) = setup_fts_workspace();

    // Search for "programming" with tag filter - should narrow down results
    let ids = query_document_ids(
        &workspace,
        &[
            "query",
            "--match",
            "programming",
            "--tag",
            "systems",
            "--output-format",
            "ids",
        ],
    );

    // Only memory-management has both "programming" in body and "systems" tag
    assert_eq!(ids.len(), 1);
    assert!(ids.contains(&"memory-management".to_string()));
}

#[test]
fn test_fts_search_common_term() {
    let (_temp, workspace) = setup_fts_workspace();

    // Search for "programming" - appears in multiple documents
    let ids = query_document_ids(
        &workspace,
        &["query", "--match", "programming", "--output-format", "ids"],
    );

    // Should match rust-programming, python-tutorial, and memory-management
    assert!(ids.len() >= 2);
    assert!(ids.contains(&"rust-programming".to_string()));
    assert!(ids.contains(&"memory-management".to_string()));
}
