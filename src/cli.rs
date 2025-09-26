use std::string::ToString;
use std::{fmt, path::PathBuf};

use clap::{Parser, Subcommand};
use env_logger::Env;

///    ____      __
///   /_  / ___ / /_
///    / /_/ -_) __/
///   /___/\__/\__/
/// Your PKM Assistant             
#[derive(Parser)]
#[command(version, about, long_about, verbatim_doc_comment)]
pub struct CliInterface {
    /// Tell zet to look for a .zet directory in `root`.
    /// If no such directory could be found, zet will look in user data
    #[arg(long)]
    pub root: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Parse { path: PathBuf },
    Init,
    Lsp,
    Format,
}
