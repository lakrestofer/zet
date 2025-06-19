use std::path::PathBuf;

use clap::{Parser, Subcommand};

///    ____      __
///   /_  / ___ / /_
///    / /_/ -_) __/
///   /___/\__/\__/
/// Your PKM Assistant             
#[derive(Parser)]
#[command(version, about, long_about, verbatim_doc_comment)]
pub struct CliInterfacce {
    /// Tell zet to look for a .zet directory in `root`.
    /// If no such directory could be found, zet will look in user data
    #[arg(long)]
    pub root: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Init,
    Lsp,
    Format,
}
