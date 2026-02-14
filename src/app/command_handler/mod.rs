use zet::core::parser::FrontMatterFormat;

pub mod create;
pub mod index;
pub mod init;
pub mod lsp;
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
                group: Default::default(),
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
                group: Default::default(),
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
                sort_configs,
                limit,
                output_format,
                delimiter,
                pretty,
                template,
            )?;
        }
        Command::Lsp => {}
        Command::Format => todo!(),
        Command::Create {
            title,
            content,
            group,
            template,
            stdin,
            data_json,
            data_toml,
            data_json_path,
            data_toml_path,
        } => create::handle_command(
            root,
            title,
            content,
            group,
            template,
            stdin,
            data_json,
            data_toml,
            data_json_path,
            data_toml_path,
        )?,
    }
    Ok(())
}
