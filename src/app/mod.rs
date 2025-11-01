pub mod cli;

pub mod error_handling {
    pub type Result<T> = color_eyre::Result<T>;
}

pub mod preamble {
    pub use crate::app::error_handling::*;
    pub use std::path::PathBuf;
}

pub mod commands {
    use clap::Subcommand;
    use std::path::PathBuf;
    #[derive(Subcommand, Debug)]
    pub enum Command {
        Parse {
            path: PathBuf,
        },
        Init {
            root: Option<PathBuf>,
            #[arg(long, default_value_t = false)]
            force: bool,
        },
        Lsp,
        Format,
    }
}
pub mod command_handler {
    use zet::core::parser::FrontMatterFormat;

    use crate::app::commands::Command;
    use crate::app::preamble::*;

    pub fn handle_command(command: Command, root: Option<PathBuf>) -> Result<()> {
        match command {
            Command::Init { root, force } => init::handle_command(root, force)?,
            command => {
                let config = zet::config::Config {
                    root: zet::core::paths::resolve_root(root)?,
                    front_matter_format: FrontMatterFormat::Toml,
                };

                log::debug!("root: {:?}", config.root);

                match command {
                    Command::Parse { path } => parse::handle_command(config, path)?,
                    Command::Lsp => todo!(),
                    Command::Format => todo!(),
                    _ => unreachable!(),
                }
            }
        }
        Ok(())
    }

    pub mod parse {
        use zet::{config::Config, core::parser::FrontMatterParser};

        use crate::app::preamble::*;

        pub fn handle_command(config: Config, path: PathBuf) -> Result<()> {
            log::debug!("parsing {:?}", path);

            let frontmatter_parser = FrontMatterParser::new(config.front_matter_format);
            let content_parser = zet::core::parser::DocumentParser::new();

            let document = std::fs::read_to_string(path)?;

            _ = zet::core::parser::parse(frontmatter_parser, content_parser, document)?;

            Ok(())
        }
    }

    pub mod init {
        use crate::app::preamble::*;
        use color_eyre::eyre::eyre;
        use normalize_path::NormalizePath;
        use resolve_path::PathResolveExt;
        use std::path::PathBuf;
        use zet::core::paths::{app_work_dir, db_dir};

        pub fn handle_command(root: Option<PathBuf>, force: bool) -> Result<()> {
            let root = root.unwrap_or(std::env::current_dir()?);
            let root: PathBuf = root.try_resolve()?.into_owned().normalize();

            let work_dir = app_work_dir(&root); // .zet
            let db_dir = db_dir(&root); // .zet/db.sqlite

            // handle if the path already exists
            if work_dir.exists() {
                if !force {
                    log::error!("{:?} already exists! specify --force to reinit", work_dir);
                    return Err(eyre!("could not initialize {:?}", work_dir));
                }
                std::fs::remove_dir_all(&work_dir)?;
            }
            std::fs::create_dir_all(&work_dir)?;

            if db_dir.is_file() {
                std::fs::remove_file(&db_dir)?;
            }

            // create and execute migrations on directory
            let _ = zet::core::db::DB::open(db_dir)?;

            // TODO, write default configuration file

            Ok(())
        }
    }
}
