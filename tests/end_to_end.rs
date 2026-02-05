use assert_fs::TempDir;

fn init_test() -> TempDir {
    let cwd = assert_fs::TempDir::new().unwrap();
    return cwd;
}


