pub mod cli;
pub mod command_handler;
pub mod commands;
pub mod error_handling;

pub mod preamble {
    pub use crate::app::commands::Command;
    pub use crate::app::error_handling::*;
    pub use std::path::PathBuf;
}
