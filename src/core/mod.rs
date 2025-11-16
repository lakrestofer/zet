pub mod collection;
pub mod db;
pub mod hasher;
pub mod parser;
pub mod paths;
pub mod slug;
pub mod types;

pub mod util {
    use std::path::Path;
    use std::path::PathBuf;

    pub fn paths_to_relative(root: &Path, paths: &Vec<PathBuf>) -> Vec<PathBuf> {
        let disk_paths_relative: Vec<PathBuf> = paths
            .iter()
            .map(|p| p.strip_prefix(root).unwrap().to_owned())
            .collect();
        log::debug!("relative_disk_paths: {:?}", disk_paths_relative);
        disk_paths_relative
    }
    pub fn paths_to_id(paths: &Vec<PathBuf>) -> Vec<String> {
        let disk_path_slugs: Vec<String> = paths
            .iter()
            .map(|p| crate::core::slug::slugify(p.to_str().unwrap()))
            .collect();
        log::debug!("disk_path_slugs : {:?}", disk_path_slugs);
        disk_path_slugs
    }
}
