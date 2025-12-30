use clap::Subcommand;
use std::path::PathBuf;
#[derive(Subcommand, Debug)]
pub enum Command {
    Parse {
        path: PathBuf,
    },
    /// Reindex the collection. Parsing any new/updated files and updating the cache.
    Index {
        #[arg(long, default_value_t = false)]
        force: bool,
    },
    Init {
        root: Option<PathBuf>,
        #[arg(long, default_value_t = false)]
        force: bool,
    },
    Lsp,
    Format,
    RawParse {
        path: PathBuf,
    },
}
