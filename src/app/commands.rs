use clap::Subcommand;
use std::path::PathBuf;
#[derive(Subcommand, Debug)]
pub enum Command {
    Parse {
        path: PathBuf,
        #[arg(long, default_value_t = false)]
        pretty_print: bool,
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
    Query {
        #[arg(long, value_delimiter = ',')]
        tag: Vec<String>,

        #[arg(long, value_delimiter = ',')]
        exclude: Vec<String>,

        #[arg(long)]
        created: String,

        #[arg(long, value_delimiter = ',')]
        /// List all notes that link to the notes with the given ids
        link_to: Vec<String>,

        #[arg(long, value_delimiter = ',')]
        /// List all notes reachable from the set of notes with the given ids
        link_from: Vec<String>,

        #[arg(long, value_delimiter = ',')]
        r#match: Vec<String>,

        #[arg(long)]
        format: Option<String>,
    },
    Lsp,
    Format,
    RawParse {
        path: PathBuf,
    },
}
