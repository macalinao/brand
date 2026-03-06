use std::path::PathBuf;

use anyhow::{Context, Result};
use bpaf::{Bpaf, ShellComp};

use crate::config::{BrandConfig, brand_config};
use crate::generate;

#[derive(Debug, Clone, Bpaf)]
pub struct GenerateArgs {
    /// Path to brand.toml config file
    #[bpaf(long, fallback(PathBuf::from("brand.toml")), complete_shell(ShellComp::File { mask: Some("*.toml") }))]
    config: PathBuf,

    #[bpaf(external(brand_config))]
    overrides: BrandConfig,
}

pub fn run(args: GenerateArgs) -> Result<()> {
    let config_path = args
        .config
        .canonicalize()
        .with_context(|| format!("Config file not found: {}", args.config.display()))?;
    let base_dir = config_path
        .parent()
        .context("Config file has no parent directory")?;

    let mut config = BrandConfig::load(&config_path)?;
    config.merge(args.overrides);
    config.resolve_paths(base_dir)?;
    generate::run(&config)
}
