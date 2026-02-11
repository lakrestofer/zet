#![feature(trait_alias)]
pub mod core;

pub const APP_NAME: &str = "zet";
pub const DB_NAME: &str = "db.sqlite";
pub const CONFIG_NAME: &str = "config.toml";
pub const APP_ENV_PREFIX: &str = "ZET_";

pub mod preamble {
    pub use crate::result::*;
    pub use crate::{APP_NAME, DB_NAME};
}

pub mod result {
    pub type Result<T> = color_eyre::Result<T>;
}

pub mod config {
    use std::collections::HashMap;
    use std::path::Path;

    use figment::Figment;
    use figment::providers::{Env, Format, Toml};
    use serde::{Deserialize, Serialize};

    use crate::APP_ENV_PREFIX;
    use crate::core::parser::FrontMatterFormat;
    use crate::core::{collection_config_file, global_config_file};
    use crate::result::Result;

    #[derive(Default, Debug, Serialize, Deserialize)]
    pub struct GroupConfig {
        /// Paths relative to collection root that belong to this group
        pub directories: Vec<String>,
        /// Template name or path. If it contains '.', treated as path in .zet/templates/<path>.
        /// Otherwise tries .zet/templates/<name>.md
        pub template: Option<String>,
    }

    #[derive(Default, Debug, Serialize, Deserialize)]
    pub struct Config {
        // pub root: PathBuf,
        #[serde(default)]
        pub front_matter_format: FrontMatterFormat,
        #[serde(default)]
        pub group: HashMap<String, GroupConfig>,
    }

    impl Config {
        pub fn resolve(root: &Path) -> Result<Config> {
            Ok(Figment::new()
                // global config
                .merge(Toml::file(global_config_file()))
                .merge(Toml::file(collection_config_file(root)))
                .merge(Env::prefixed(APP_ENV_PREFIX))
                .extract()?)
        }
    }
}
