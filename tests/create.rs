mod helpers;

use helpers::{cli::run_cli_cmd, setup_temp_workspace};
use std::fs;
use std::path::Path;

fn init_workspace(workspace: &Path) {
    run_cli_cmd(&["init"], workspace).assert().success();
}

fn get_stdout(assert: &assert_cmd::assert::Assert) -> String {
    String::from_utf8(assert.get_output().stdout.clone()).unwrap()
}

fn get_stderr(assert: &assert_cmd::assert::Assert) -> String {
    String::from_utf8(assert.get_output().stderr.clone()).unwrap()
}

// ---- Happy Path ----

#[test]
fn test_create_basic() {
    let (_temp, workspace) = setup_temp_workspace();
    init_workspace(&workspace);

    let assert = run_cli_cmd(&["create", "My First Note"], &workspace)
        .assert()
        .success();
    let stdout = get_stdout(&assert);
    let path_str = stdout.trim();

    let path = Path::new(path_str);
    assert!(path.exists(), "output file should exist: {path_str}");
    assert_eq!(path.file_name().unwrap(), "my-first-note.md");

    let content = fs::read_to_string(path).unwrap();
    assert!(content.contains("id: my-first-note"), "missing id field");
    assert!(content.contains("title: My First Note"), "missing title field");
}

#[test]
fn test_create_with_content_arg() {
    let (_temp, workspace) = setup_temp_workspace();
    init_workspace(&workspace);

    let assert = run_cli_cmd(&["create", "My Note", "Hello world"], &workspace)
        .assert()
        .success();
    let path_str = get_stdout(&assert);
    let path_str = path_str.trim();

    let content = fs::read_to_string(path_str).unwrap();
    assert!(content.contains("Hello world"), "missing inline content");
}

#[test]
fn test_create_with_stdin() {
    let (_temp, workspace) = setup_temp_workspace();
    init_workspace(&workspace);

    let assert = run_cli_cmd(&["create", "Stdin Note", "--stdin"], &workspace)
        .write_stdin("piped\n")
        .assert()
        .success();
    let path_str = get_stdout(&assert);
    let path_str = path_str.trim();

    let content = fs::read_to_string(path_str).unwrap();
    assert!(content.contains("piped"), "missing stdin content");
}

#[test]
fn test_create_date_is_injected() {
    let (_temp, workspace) = setup_temp_workspace();
    init_workspace(&workspace);

    // Write a custom template that emits the date field
    let templates_dir = workspace.join(".zet/templates");
    fs::create_dir_all(&templates_dir).unwrap();
    fs::write(
        templates_dir.join("dated.md"),
        "---\nid: {{ id }}\ntitle: {{ title }}\ndate: {{ date }}\n---\n\n{{ content }}\n",
    )
    .unwrap();

    let assert = run_cli_cmd(
        &["create", "Dated Note", "--template", "dated.md"],
        &workspace,
    )
    .assert()
    .success();
    let path_str = get_stdout(&assert);
    let path_str = path_str.trim();

    let content = fs::read_to_string(path_str).unwrap();

    // Verify the date field was rendered in YYYY-MM-DD format
    let has_date_field = content.lines().any(|line| {
        if let Some(val) = line.strip_prefix("date: ") {
            val.len() == 10
                && val.as_bytes().get(4) == Some(&b'-')
                && val.as_bytes().get(7) == Some(&b'-')
                && val.as_bytes()[0..4].iter().all(|b| b.is_ascii_digit())
        } else {
            false
        }
    });
    assert!(
        has_date_field,
        "date should be injected in YYYY-MM-DD format\ncontent:\n{content}"
    );
}

#[test]
fn test_create_with_explicit_template() {
    let (_temp, workspace) = setup_temp_workspace();
    init_workspace(&workspace);

    let templates_dir = workspace.join(".zet/templates");
    fs::create_dir_all(&templates_dir).unwrap();
    fs::write(
        templates_dir.join("custom.md"),
        "# {{ title }}\nCustom template content\n",
    )
    .unwrap();

    let assert = run_cli_cmd(
        &["create", "Template Note", "--template", "custom.md"],
        &workspace,
    )
    .assert()
    .success();
    let path_str = get_stdout(&assert);
    let path_str = path_str.trim();

    assert!(Path::new(path_str).exists(), "output file should exist");
    let content = fs::read_to_string(path_str).unwrap();
    assert!(
        content.contains("Custom template content"),
        "explicit template was not used"
    );
}

#[test]
fn test_create_with_group() {
    let (_temp, workspace) = setup_temp_workspace();
    init_workspace(&workspace);

    fs::write(
        workspace.join(".zet/config.toml"),
        "[group.journal]\ndirectories = [\"journal\"]\ntemplate = \"journal.md\"\n",
    )
    .unwrap();

    let templates_dir = workspace.join(".zet/templates");
    fs::create_dir_all(&templates_dir).unwrap();
    fs::write(
        templates_dir.join("journal.md"),
        "---\nid: {{ id }}\ntitle: {{ title }}\n---\n\nJournal entry\n{{ content }}\n",
    )
    .unwrap();

    fs::create_dir_all(workspace.join("journal")).unwrap();

    let assert = run_cli_cmd(
        &["create", "My Journal", "--group", "journal"],
        &workspace,
    )
    .assert()
    .success();
    let path_str = get_stdout(&assert);
    let path_str = path_str.trim();

    let path = Path::new(path_str);
    assert!(path.exists(), "output file should exist");
    assert!(
        path.parent().unwrap().ends_with("journal"),
        "file should be in journal/ directory, got: {path_str}"
    );
    let content = fs::read_to_string(path).unwrap();
    assert!(content.contains("Journal entry"), "group template was not used");
}

#[test]
fn test_create_group_creates_directory() {
    let (_temp, workspace) = setup_temp_workspace();
    init_workspace(&workspace);

    fs::write(
        workspace.join(".zet/config.toml"),
        "[group.journal]\ndirectories = [\"journal\"]\n",
    )
    .unwrap();

    // journal/ does NOT exist yet
    assert!(!workspace.join("journal").exists());

    run_cli_cmd(&["create", "Auto Dir Note", "--group", "journal"], &workspace)
        .assert()
        .success();

    assert!(
        workspace.join("journal").exists(),
        "journal/ directory should be auto-created"
    );
}

#[test]
fn test_create_cwd_group_matching() {
    let (_temp, workspace) = setup_temp_workspace();
    init_workspace(&workspace);

    fs::write(
        workspace.join(".zet/config.toml"),
        "[group.journal]\ndirectories = [\"journal\"]\ntemplate = \"journal.md\"\n",
    )
    .unwrap();

    let templates_dir = workspace.join(".zet/templates");
    fs::create_dir_all(&templates_dir).unwrap();
    fs::write(
        templates_dir.join("journal.md"),
        "---\nid: {{ id }}\ntitle: {{ title }}\n---\n\nGroup matched\n{{ content }}\n",
    )
    .unwrap();

    let journal_dir = workspace.join("journal");
    fs::create_dir_all(&journal_dir).unwrap();

    // Run create with CWD inside journal/ — no --group flag
    let assert = run_cli_cmd(&["create", "CWD Note"], &journal_dir)
        .assert()
        .success();
    let path_str = get_stdout(&assert);
    let path_str = path_str.trim();

    let path = Path::new(path_str);
    assert!(path.exists(), "output file should exist");
    let content = fs::read_to_string(path).unwrap();
    assert!(
        content.contains("Group matched"),
        "group template should be auto-selected from CWD\ncontent:\n{content}"
    );
}

#[test]
fn test_create_with_data_json() {
    let (_temp, workspace) = setup_temp_workspace();
    init_workspace(&workspace);

    let templates_dir = workspace.join(".zet/templates");
    fs::create_dir_all(&templates_dir).unwrap();
    fs::write(
        templates_dir.join("author.md"),
        "---\nid: {{ id }}\ntitle: {{ title }}\nauthor: {{ author }}\n---\n\n{{ content }}\n",
    )
    .unwrap();

    let assert = run_cli_cmd(
        &[
            "create",
            "Authored Note",
            "--template",
            "author.md",
            "--data-json",
            r#"{"author":"Alice"}"#,
        ],
        &workspace,
    )
    .assert()
    .success();
    let path_str = get_stdout(&assert);
    let path_str = path_str.trim();

    let content = fs::read_to_string(path_str).unwrap();
    assert!(content.contains("author: Alice"), "author not injected via --data-json");
}

#[test]
fn test_create_with_data_toml() {
    let (_temp, workspace) = setup_temp_workspace();
    init_workspace(&workspace);

    let templates_dir = workspace.join(".zet/templates");
    fs::create_dir_all(&templates_dir).unwrap();
    fs::write(
        templates_dir.join("author.md"),
        "---\nid: {{ id }}\ntitle: {{ title }}\nauthor: {{ author }}\n---\n\n{{ content }}\n",
    )
    .unwrap();

    let assert = run_cli_cmd(
        &[
            "create",
            "TOML Note",
            "--template",
            "author.md",
            "--data-toml",
            r#"author = "Bob""#,
        ],
        &workspace,
    )
    .assert()
    .success();
    let path_str = get_stdout(&assert);
    let path_str = path_str.trim();

    let content = fs::read_to_string(path_str).unwrap();
    assert!(content.contains("author: Bob"), "author not injected via --data-toml");
}

#[test]
fn test_create_with_data_json_path() {
    let (_temp, workspace) = setup_temp_workspace();
    init_workspace(&workspace);

    let templates_dir = workspace.join(".zet/templates");
    fs::create_dir_all(&templates_dir).unwrap();
    fs::write(
        templates_dir.join("author.md"),
        "---\nid: {{ id }}\ntitle: {{ title }}\nauthor: {{ author }}\n---\n\n{{ content }}\n",
    )
    .unwrap();

    let data_path = workspace.join("data.json");
    fs::write(&data_path, r#"{"author":"Carol"}"#).unwrap();

    let assert = run_cli_cmd(
        &[
            "create",
            "JSON Path Note",
            "--template",
            "author.md",
            "--data-json-path",
            data_path.to_str().unwrap(),
        ],
        &workspace,
    )
    .assert()
    .success();
    let path_str = get_stdout(&assert);
    let path_str = path_str.trim();

    let content = fs::read_to_string(path_str).unwrap();
    assert!(
        content.contains("author: Carol"),
        "author not injected via --data-json-path"
    );
}

#[test]
fn test_create_with_data_toml_path() {
    let (_temp, workspace) = setup_temp_workspace();
    init_workspace(&workspace);

    let templates_dir = workspace.join(".zet/templates");
    fs::create_dir_all(&templates_dir).unwrap();
    fs::write(
        templates_dir.join("author.md"),
        "---\nid: {{ id }}\ntitle: {{ title }}\nauthor: {{ author }}\n---\n\n{{ content }}\n",
    )
    .unwrap();

    let data_path = workspace.join("data.toml");
    fs::write(&data_path, r#"author = "Dave""#).unwrap();

    let assert = run_cli_cmd(
        &[
            "create",
            "TOML Path Note",
            "--template",
            "author.md",
            "--data-toml-path",
            data_path.to_str().unwrap(),
        ],
        &workspace,
    )
    .assert()
    .success();
    let path_str = get_stdout(&assert);
    let path_str = path_str.trim();

    let content = fs::read_to_string(path_str).unwrap();
    assert!(
        content.contains("author: Dave"),
        "author not injected via --data-toml-path"
    );
}

// ---- Error Path ----

#[test]
fn test_create_duplicate_fails() {
    let (_temp, workspace) = setup_temp_workspace();
    init_workspace(&workspace);

    run_cli_cmd(&["create", "Duplicate Note"], &workspace)
        .assert()
        .success();

    let assert = run_cli_cmd(&["create", "Duplicate Note"], &workspace)
        .assert()
        .failure();
    let stderr = get_stderr(&assert);
    assert!(
        stderr.contains("already exists"),
        "expected 'already exists' in stderr: {stderr}"
    );
}

#[test]
fn test_create_stdin_and_content_conflict() {
    let (_temp, workspace) = setup_temp_workspace();
    init_workspace(&workspace);

    let assert = run_cli_cmd(&["create", "T", "body", "--stdin"], &workspace)
        .assert()
        .failure();
    let stderr = get_stderr(&assert);
    assert!(
        stderr.contains("mutually exclusive"),
        "expected 'mutually exclusive' in stderr: {stderr}"
    );
}

#[test]
fn test_create_unknown_group_fails() {
    let (_temp, workspace) = setup_temp_workspace();
    init_workspace(&workspace);

    let assert = run_cli_cmd(&["create", "Note", "--group", "doesnotexist"], &workspace)
        .assert()
        .failure();
    let stderr = get_stderr(&assert);
    assert!(
        stderr.contains("not found in config"),
        "expected 'not found in config' in stderr: {stderr}"
    );
}

#[test]
fn test_create_missing_template_fails() {
    let (_temp, workspace) = setup_temp_workspace();
    init_workspace(&workspace);

    let assert = run_cli_cmd(
        &["create", "Note", "--template", "doesnotexist.md"],
        &workspace,
    )
    .assert()
    .failure();
    let stderr = get_stderr(&assert);
    assert!(
        stderr.contains("could not read template"),
        "expected 'could not read template' in stderr: {stderr}"
    );
}
