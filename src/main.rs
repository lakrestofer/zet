use clap::Parser;
use color_eyre::Result;
use env_logger::Env;
use zet::cli::CliInterface;
use zet::{
    ZetConfig,
    parser::{FrontMatterFormat, FrontMatterParser},
};

fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = CliInterface::parse();

    let env = Env::new().filter_or("RUST_LOG", "info");
    env_logger::init_from_env(env);

    let config = ZetConfig {
        root: zet::resolve_root(cli.root)?,
        front_matter_format: FrontMatterFormat::Toml,
    };

    log::debug!("root: {:?}", config.root);

    match cli.command {
        zet::cli::Command::Parse { path } => {
            log::debug!("parsing {:?}", path);

            let frontmatter_parser = FrontMatterParser::new(config.front_matter_format);
            let content_parser = zet::parser::DocumentParser::new();

            let document = std::fs::read_to_string(path)?;

            zet::parser::parse(frontmatter_parser, content_parser, document)?;
        }
        zet::cli::Command::Init => todo!(),
        zet::cli::Command::Lsp => todo!(),
        zet::cli::Command::Format => todo!(),
    }

    Ok(())
}
