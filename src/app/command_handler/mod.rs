use zet::core::parser::FrontMatterFormat;

pub mod index;
pub mod init;
pub mod parse;
pub mod query;
pub mod raw_parse;

use crate::app::preamble::*;
use zet::preamble::*;

pub fn handle_command(command: Command, root: Option<PathBuf>) -> Result<()> {
    match command {
        Command::Init { root, force } => init::handle_command(root, force)?,
        Command::Parse { path, pretty_print } => {
            parse::handle_command(FrontMatterFormat::Yaml, pretty_print, path)?
        }
        Command::RawParse { path } => raw_parse::handle_command(FrontMatterFormat::Yaml, path)?,
        Command::Index { force } => {
            let root = zet::core::resolve_root(root)?;
            let config = zet::config::Config {
                front_matter_format: FrontMatterFormat::Yaml,
            };
            index::handle_command(&root, config, force)?
        }
        Command::Query {
            ids,
            titles,
            paths,
            tags,
            tagless,
            exclude_list,
            exclude_by_path,
            created,
            modified,
            created_before,
            created_after,
            modified_before,
            modified_after,
            links_to,
            links_from,
            match_patterns,
            match_strategy,
            sort_configs,
            limit,
            output_format,
            delimiter,
            pretty,
            template,
        } => {
            let root = zet::core::resolve_root(root)?;

            let config = zet::config::Config {
                front_matter_format: FrontMatterFormat::Yaml,
            };

            query::handle_command(
                &root,
                config,
                ids,
                titles,
                paths,
                tags,
                tagless,
                exclude_list,
                exclude_by_path,
                created,
                modified,
                created_before,
                created_after,
                modified_before,
                modified_after,
                links_to,
                links_from,
                match_patterns,
                match_strategy,
                sort_configs,
                limit,
                output_format,
                delimiter,
                pretty,
                template,
            )?;
        }
        Command::Lsp => todo!(),
        Command::Format => todo!(),
    }
    Ok(())
}
