pub mod cli;
pub mod db;

use assert_fs::TempDir;
use color_eyre::Result;
use std::path::{Path, PathBuf};

/// Creates a temporary workspace directory for testing
pub fn setup_temp_workspace() -> (TempDir, PathBuf) {
    let temp = TempDir::new().expect("Failed to create temp directory");
    let workspace = temp.path().to_path_buf();
    (temp, workspace)
}

/// Copies a fixture directory to the temporary workspace
pub fn copy_fixture_to_temp(fixture_name: &str, temp: &TempDir) -> Result<()> {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(fixture_name);

    if !fixture_path.exists() {
        return Err(color_eyre::eyre::eyre!(
            "Fixture directory not found: {:?}",
            fixture_path
        ));
    }

    copy_dir_recursive(&fixture_path, temp.path())?;
    Ok(())
}

/// Recursively copies a directory
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    if !dst.exists() {
        std::fs::create_dir_all(dst)?;
    }

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}
