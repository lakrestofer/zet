use app::cli::argument_parser::*;
use color_eyre::Result;
use env_logger::Env;

pub mod app;

fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = ArgumentParser::parse();

    if let Some(level) = cli.level {
        env_logger::builder().filter_level(level.into()).init();
    } else {
        let env = Env::new().filter_or("RUST_LOG", "info");
        env_logger::init_from_env(env);
    }

    app::command_handler::handle_command(cli.command, cli.root)?;

    Ok(())
}
