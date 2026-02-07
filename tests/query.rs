mod helpers;

use helpers::{cli::*, *};

/// Helper to setup query test workspace
fn setup_query_workspace() -> (assert_fs::TempDir, std::path::PathBuf) {
    let (temp, workspace) = setup_temp_workspace();
    copy_fixture_to_temp("query-test", &temp).unwrap();

    run_cli_cmd(&["init"], &workspace).assert().success();
    run_cli_cmd(&["index"], &workspace).assert().success();

    (temp, workspace)
}

// =============================================================================
// Basic filters
// =============================================================================

#[test]
fn test_query_all_documents() {
    let (_temp, workspace) = setup_query_workspace();

    let ids = query_document_ids(&workspace, &["query", "--output-format", "ids"]);

    assert_eq!(ids.len(), 5);
}

#[test]
fn test_query_by_id() {
    let (_temp, workspace) = setup_query_workspace();

    let ids = query_document_ids(&workspace, &["query", "--id", "alpha", "--output-format", "ids"]);

    assert_eq!(ids.len(), 1);
    assert!(ids.contains(&"alpha".to_string()));
}

#[test]
fn test_query_by_multiple_ids() {
    let (_temp, workspace) = setup_query_workspace();

    let ids = query_document_ids(
        &workspace,
        &["query", "--id", "alpha,beta", "--output-format", "ids"],
    );

    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&"alpha".to_string()));
    assert!(ids.contains(&"beta".to_string()));
}

#[test]
fn test_query_by_title() {
    let (_temp, workspace) = setup_query_workspace();

    let ids = query_document_ids(
        &workspace,
        &["query", "--title", "Alpha Document", "--output-format", "ids"],
    );

    assert_eq!(ids.len(), 1);
    assert!(ids.contains(&"alpha".to_string()));
}

#[test]
fn test_query_by_path() {
    let (_temp, workspace) = setup_query_workspace();

    let ids = query_document_ids(
        &workspace,
        &["query", "--path", "alpha.md", "--output-format", "ids"],
    );

    assert_eq!(ids.len(), 1);
    assert!(ids.contains(&"alpha".to_string()));
}

// =============================================================================
// Tag filters
// =============================================================================

#[test]
fn test_query_by_single_tag() {
    let (_temp, workspace) = setup_query_workspace();

    let ids = query_document_ids(
        &workspace,
        &["query", "--tag", "work", "--output-format", "ids"],
    );

    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&"alpha".to_string()));
    assert!(ids.contains(&"beta".to_string()));
}

#[test]
fn test_query_by_multiple_tags_and() {
    let (_temp, workspace) = setup_query_workspace();

    // Multiple tags use AND semantics: only beta has both "work" AND "personal"
    let ids = query_document_ids(
        &workspace,
        &["query", "--tag", "work,personal", "--output-format", "ids"],
    );

    assert_eq!(ids.len(), 1);
    assert!(ids.contains(&"beta".to_string()));
}

#[test]
fn test_query_tagless() {
    let (_temp, workspace) = setup_query_workspace();

    let ids = query_document_ids(&workspace, &["query", "--tagless", "--output-format", "ids"]);

    assert_eq!(ids.len(), 1);
    assert!(ids.contains(&"delta".to_string()));
}

// =============================================================================
// Exclusion filters
// =============================================================================

#[test]
fn test_query_exclude_by_id() {
    let (_temp, workspace) = setup_query_workspace();

    let ids = query_document_ids(
        &workspace,
        &["query", "--exclude", "alpha", "--output-format", "ids"],
    );

    assert_eq!(ids.len(), 4);
    assert!(!ids.contains(&"alpha".to_string()));
}

#[test]
fn test_query_exclude_by_path() {
    let (_temp, workspace) = setup_query_workspace();

    let ids = query_document_ids(
        &workspace,
        &["query", "--exclude-by-path", "alpha.md", "--output-format", "ids"],
    );

    assert_eq!(ids.len(), 4);
    assert!(!ids.contains(&"alpha".to_string()));
}

// =============================================================================
// Link filters
// =============================================================================

#[test]
fn test_query_links_to() {
    let (_temp, workspace) = setup_query_workspace();

    // --links-to gamma returns documents that link TO gamma (alpha and beta)
    let ids = query_document_ids(
        &workspace,
        &["query", "--links-to", "gamma", "--output-format", "ids"],
    );

    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&"alpha".to_string()));
    assert!(ids.contains(&"beta".to_string()));
}

#[test]
fn test_query_links_from() {
    let (_temp, workspace) = setup_query_workspace();

    // --links-from alpha returns documents that are linked FROM alpha (beta and gamma)
    let ids = query_document_ids(
        &workspace,
        &["query", "--links-from", "alpha", "--output-format", "ids"],
    );

    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&"beta".to_string()));
    assert!(ids.contains(&"gamma".to_string()));
}

// =============================================================================
// Sorting and limiting
// =============================================================================

#[test]
fn test_query_sort_by_id_asc() {
    let (_temp, workspace) = setup_query_workspace();

    let ids = query_document_ids(
        &workspace,
        &["query", "--sort", "id+", "--output-format", "ids"],
    );

    assert_eq!(ids.len(), 5);
    assert_eq!(ids[0], "alpha");
    assert_eq!(ids[1], "beta");
    assert_eq!(ids[2], "delta");
    assert_eq!(ids[3], "epsilon");
    assert_eq!(ids[4], "gamma");
}

#[test]
fn test_query_sort_by_id_desc() {
    let (_temp, workspace) = setup_query_workspace();

    let ids = query_document_ids(
        &workspace,
        &["query", "--sort", "id-", "--output-format", "ids"],
    );

    assert_eq!(ids.len(), 5);
    assert_eq!(ids[0], "gamma");
    assert_eq!(ids[1], "epsilon");
    assert_eq!(ids[2], "delta");
    assert_eq!(ids[3], "beta");
    assert_eq!(ids[4], "alpha");
}

#[test]
fn test_query_limit() {
    let (_temp, workspace) = setup_query_workspace();

    let ids = query_document_ids(
        &workspace,
        &["query", "--limit", "2", "--output-format", "ids"],
    );

    assert_eq!(ids.len(), 2);
}

#[test]
fn test_query_sort_and_limit() {
    let (_temp, workspace) = setup_query_workspace();

    let ids = query_document_ids(
        &workspace,
        &["query", "--sort", "id+", "--limit", "2", "--output-format", "ids"],
    );

    assert_eq!(ids.len(), 2);
    assert_eq!(ids[0], "alpha");
    assert_eq!(ids[1], "beta");
}

// =============================================================================
// Combined filters
// =============================================================================

#[test]
fn test_query_tag_and_exclude() {
    let (_temp, workspace) = setup_query_workspace();

    // Documents with tag "work" but excluding alpha
    let ids = query_document_ids(
        &workspace,
        &["query", "--tag", "work", "--exclude", "alpha", "--output-format", "ids"],
    );

    assert_eq!(ids.len(), 1);
    assert!(ids.contains(&"beta".to_string()));
}

#[test]
fn test_query_links_to_and_tag() {
    let (_temp, workspace) = setup_query_workspace();

    // Documents that link to gamma AND have tag "work"
    let ids = query_document_ids(
        &workspace,
        &["query", "--links-to", "gamma", "--tag", "work", "--output-format", "ids"],
    );

    // Both alpha and beta link to gamma and have tag "work"
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&"alpha".to_string()));
    assert!(ids.contains(&"beta".to_string()));
}
