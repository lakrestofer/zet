
pub mod core;

pub const APP_NAME: &str = "zet";
pub const DB_NAME: &str = "db.sqlite";

pub mod preamble {
    pub use crate::result::*;
    pub use crate::{APP_NAME, DB_NAME};
}

pub mod result {
    pub type Result<T> = color_eyre::Result<T>;
}

pub mod config {
    use std::path::PathBuf;

    use crate::core::parser::FrontMatterFormat;

    pub struct Config {
        pub root: PathBuf,
        pub front_matter_format: FrontMatterFormat,
    }
}
