use assert_cmd::{Command, cargo};
use std::path::Path;

/// Runs a CLI command in the given working directory
pub fn run_cli_cmd(args: &[&str], cwd: &Path) -> Command {
    let mut cmd = Command::new(cargo::cargo_bin!("zet"));
    cmd.current_dir(cwd);
    cmd.args(args);
    cmd
}

/// Runs a command and asserts success
pub fn assert_success(cmd: &mut Command) -> assert_cmd::assert::Assert {
    cmd.assert().success()
}

/// Queries document IDs using the CLI
pub fn query_document_ids(workspace: &Path, args: &[&str]) -> Vec<String> {
    let mut cmd = run_cli_cmd(args, workspace);
    let output = cmd.output().expect("Failed to execute query command");

    if !output.status.success() {
        panic!(
            "Query command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}
