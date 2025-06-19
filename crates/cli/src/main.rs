use clap::Parser;
use color_eyre::Result;
use env_logger::Env;
use zet::cli_interface::CliInterfacce;

fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = CliInterfacce::parse();

    let env = Env::new().filter_or("RUST_LOG", "info");
    env_logger::init_from_env(env);
    Ok(())
}
