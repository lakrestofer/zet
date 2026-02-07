use chumsky::Parser;
use chumsky::extra;
use chumsky::prelude::choice;
use chumsky::prelude::just;
use clap::Subcommand;
use clap::ValueEnum;
use color_eyre::eyre::eyre;
use jiff::Timestamp;
use std::path::PathBuf;
use zet::core::date_parser::NaturalDateParser;
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
        /// clear the cache and reindex the entire collection
        force: bool,
    },
    Init {
        root: Option<PathBuf>,
        #[arg(long, default_value_t = false)]
        force: bool,
    },
    Query {
        #[arg(long = "id", value_delimiter = ',')]
        ids: Vec<String>,
        #[arg(long = "title", value_delimiter = ',')]
        titles: Vec<String>,
        #[arg(long = "path", value_delimiter = ',')]
        paths: Vec<String>,
        ////////////////////////////////////////////////////////////
        // tags
        ////////////////////////////////////////////////////////////
        #[arg(long = "tag", value_delimiter = ',')]
        /// list notes that includes `tag`
        tags: Vec<String>,
        #[arg(long)]
        /// list notes that have no `tag`
        tagless: bool,

        ////////////////////////////////////////////////////////////
        // explicit sets of ids
        ////////////////////////////////////////////////////////////
        #[arg(long = "exclude", value_delimiter = ',')]
        /// list notes in the set described by `exclude`
        exclude_list: Vec<String>,
        #[arg(long, value_delimiter = ',')]
        exclude_by_path: Vec<String>,

        ////////////////////////////////////////////////////////////
        // created and modified timestamps
        ////////////////////////////////////////////////////////////
        #[arg(long, value_parser=natural_language_parser)]
        created: Option<Timestamp>,
        #[arg(long, value_parser=natural_language_parser)]
        modified: Option<Timestamp>,
        #[arg(long, value_parser=natural_language_parser)]
        created_before: Option<Timestamp>,
        #[arg(long, value_parser=natural_language_parser)]
        created_after: Option<Timestamp>,
        #[arg(long, value_parser=natural_language_parser)]
        modified_before: Option<Timestamp>,
        #[arg(long, value_parser=natural_language_parser)]
        modified_after: Option<Timestamp>,

        ////////////////////////////////////////////////////////////
        // links
        ////////////////////////////////////////////////////////////
        #[arg(long, value_delimiter = ',')]
        /// List all notes that link to the notes with the given ids
        links_to: Vec<String>,
        #[arg(long, value_delimiter = ',')]
        /// List all notes that link from the set of notes with the given ids
        links_from: Vec<String>,
        // #[arg(long)]
        // /// List all notes that link from the set of notes with the given ids
        // recursive: bool,

        ////////////////////////////////////////////////////////////
        // patterns
        ////////////////////////////////////////////////////////////
        #[arg(long = "match", value_delimiter = ',')]
        match_patterns: Vec<String>,

        #[arg(long, default_value_t=MatchStrategy::FTS)]
        match_strategy: MatchStrategy,

        ////////////////////////////////////////////////////////////
        // output options
        ////////////////////////////////////////////////////////////
        #[arg(long="sort", value_delimiter = ',', value_parser=parse_sort_option)]
        /// sort the result by some configuration
        sort_configs: Vec<SortConfig>,
        #[arg(long)]
        /// limit the number of results returned
        limit: Option<usize>,
        #[arg(long)]
        /// how each document should be formatted
        output_format: OutputFormat,
        #[arg(long)]
        /// separator between each formatted document
        delimiter: Option<String>,
        #[arg(long)]
        /// whether json output should be pretty printed or not
        pretty: bool,
        #[arg(long)]
        template: Option<String>,
    },
    Lsp,
    Format,
    RawParse {
        path: PathBuf,
    },
}

#[derive(Debug, Clone)]
pub struct SortConfig {
    pub by: SortByOption,
    pub order: SortOrder,
}

#[derive(Debug, Clone)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum SortByOption {
    Modified,
    Created,
    Id,
    Path,
    Title,
    // Random,
    // WordCount,
}

fn parse_sort_option<'src>(input: &'src str) -> zet::result::Result<SortConfig> {
    let input = input.to_lowercase();

    let parser = choice((
        just::<_, _, extra::Default>("modified").to(SortByOption::Modified),
        just("created").to(SortByOption::Created),
        just("modified").to(SortByOption::Modified),
        just("created").to(SortByOption::Created),
        just("id").to(SortByOption::Id),
        just("path").to(SortByOption::Path),
        just("title").to(SortByOption::Title),
        // just("random").to(SortByOption::Random),
        // just("wordcount").to(SortByOption::WordCount),
    ))
    .then(
        choice((
            just("+").to(SortOrder::Ascending),
            just("-").to(SortOrder::Descending),
        ))
        .or_not(),
    );

    let res = parser
        .parse(input.as_str())
        .into_result()
        .map_err(|_| eyre!("could not parse sort argument"))?;

    let (by, order) = match res {
        (by, Some(ord)) => (by, ord),
        // (SortByOption::Random, _) => (res.0, SortOrder::Ascending),
        (by, None) => {
            let ord = match by {
                SortByOption::Modified => SortOrder::Descending,
                SortByOption::Created => SortOrder::Descending,
                SortByOption::Id => SortOrder::Ascending,
                SortByOption::Path => SortOrder::Ascending,
                SortByOption::Title => SortOrder::Ascending,
                // SortByOption::WordCount => SortOrder::Ascending,
                // SortByOption::Random => unreachable!(),
            };
            (by, ord)
        }
    };

    Ok(SortConfig { by, order })
}

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    Ids,
    Json,
    Template,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum MatchStrategy {
    FTS,
    Exact,
    RegularExpr,
}

impl ToString for MatchStrategy {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}
impl ToString for OutputFormat {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

// private

fn natural_language_parser(input: &str) -> zet::result::Result<Timestamp> {
    NaturalDateParser::parse(input, jiff::Timestamp::now())
        .map_err(|e| eyre!("invalid date expression: {:?}", e))
}
