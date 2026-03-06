use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use bpaf::{Bpaf, ShellComp};
use ignore::Walk;

use crate::config::{BrandConfig, brand_config};
use crate::generate;

#[derive(Debug, Clone, Bpaf)]
pub struct GenerateArgs {
    /// Find and generate all brand.toml files in the repository
    #[bpaf(long, switch)]
    workspace: bool,

    /// Path to brand.toml config file
    #[bpaf(long, fallback(PathBuf::from("brand.toml")), complete_shell(ShellComp::File { mask: Some("*.toml") }))]
    config: PathBuf,

    #[bpaf(external(brand_config))]
    overrides: BrandConfig,
}

pub fn run(args: &GenerateArgs) -> Result<()> {
    if args.workspace {
        return run_workspace(&args.overrides);
    }
    run_single(&args.config, &args.overrides)
}

fn run_single(config_path: &Path, overrides: &BrandConfig) -> Result<()> {
    let config_path = config_path
        .canonicalize()
        .with_context(|| format!("Config file not found: {}", config_path.display()))?;
    let base_dir = config_path
        .parent()
        .context("Config file has no parent directory")?;

    let mut config = BrandConfig::load(&config_path)?;
    config.merge(overrides.clone());
    config.resolve_paths(base_dir)?;
    generate::run(&config)
}

fn run_workspace(overrides: &BrandConfig) -> Result<()> {
    let cwd = std::env::current_dir().context("Failed to get current directory")?;

    let mut configs: Vec<PathBuf> = Walk::new(&cwd)
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name() == "brand.toml")
        .map(|entry| {
            let path = entry.into_path();
            let relative = path.strip_prefix(&cwd).unwrap_or(&path);
            eprintln!("Found: {}", relative.display());
            path
        })
        .collect();
    configs.sort();

    if configs.is_empty() {
        bail!("No brand.toml files found");
    }

    eprintln!();
    let mut errors = Vec::new();
    for config_path in &configs {
        let relative = config_path.strip_prefix(&cwd).unwrap_or(config_path);
        eprintln!("Generating: {}", relative.display());
        if let Err(e) = run_single(config_path, overrides) {
            eprintln!("  Error: {e:#}");
            errors.push((relative.to_path_buf(), e));
        }
    }

    if errors.is_empty() {
        eprintln!("Generated {} brand(s)", configs.len());
        Ok(())
    } else {
        bail!(
            "Failed to generate {} of {} brand(s)",
            errors.len(),
            configs.len()
        )
    }
}
