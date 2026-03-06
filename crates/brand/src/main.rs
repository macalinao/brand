use std::process::ExitCode;

use bpaf::Bpaf;
use tracing_subscriber::EnvFilter;

mod commands;
mod config;
mod fonts;
mod generate;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options, version, fallback_to_usage, generate(cli))]
#[allow(clippy::upper_case_acronyms)]
/// Generate a brand icon set from an SVG icon, brand name, and font.
///
/// Produces favicon.ico, icon PNGs, logo SVGs (dark/light), apple touch icons,
/// and site.webmanifest.
struct CLI {
    #[bpaf(external(commands))]
    command: Commands,
}

#[derive(Debug, Clone, Bpaf)]
#[allow(clippy::large_enum_variant)]
enum Commands {
    /// Generate brand assets from a brand.toml configuration
    #[bpaf(command("generate"))]
    Generate(#[bpaf(external(commands::generate::generate_args))] commands::generate::GenerateArgs),
    /// Interactively create a brand.toml configuration
    #[bpaf(command("init"))]
    Init(#[bpaf(external(commands::init::init_args))] commands::init::InitArgs),
}

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let opts = cli().run();

    let result = match &opts.command {
        Commands::Generate(args) => commands::generate::run(args.clone()),
        Commands::Init(args) => commands::init::run(args),
    };

    if let Err(e) = result {
        eprintln!("Error: {e:#}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
