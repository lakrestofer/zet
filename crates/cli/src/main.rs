use clap::Parser;
use color_eyre::Result;
use env_logger::Env;
use zet::cli_interface::CliInterfacce;
use zetlib::{
    ZetConfig,
    parser::{FrontMatterFormat, FrontMatterParser},
};

fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = CliInterfacce::parse();

    let config = ZetConfig {
        root: std::env::current_dir()?,
        front_matter_format: FrontMatterFormat::Toml,
    };

    let env = Env::new().filter_or("RUST_LOG", "info");
    env_logger::init_from_env(env);

    match cli.command {
        zet::cli_interface::Command::Parse { path } => {
            log::debug!("parsing {:?}", path);

            let frontmatter_parser = FrontMatterParser::new(config.front_matter_format);
            let content_parser = zetlib::parser::DocumentParser::new();

            let document = std::fs::read_to_string(path)?;

            zetlib::parser::parse(frontmatter_parser, content_parser, document)?;
        }
        zet::cli_interface::Command::Init => todo!(),
        zet::cli_interface::Command::Lsp => todo!(),
        zet::cli_interface::Command::Format => todo!(),
    }

    Ok(())
}
