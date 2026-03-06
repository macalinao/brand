use std::path::PathBuf;

use anyhow::{Context, Result};
use bpaf::{Bpaf, ShellComp};
use dialoguer::{Input, theme::ColorfulTheme};

use crate::config::{AssetsConfig, BrandConfig, IconConfig, ThemeConfig};
use brand_usvg_options::UsvgOptions;

/// Interactively initialize a brand.toml configuration.
#[derive(Debug, Clone, Bpaf)]
pub struct InitArgs {
    /// Output path for brand.toml
    #[bpaf(short, long, fallback(PathBuf::from("brand.toml")), complete_shell(ShellComp::File { mask: Some("*.toml") }))]
    output: PathBuf,
}

pub fn run(args: &InitArgs) -> Result<()> {
    let theme = ColorfulTheme::default();

    let name: String = Input::with_theme(&theme)
        .with_prompt("Brand name")
        .interact_text()
        .context("Failed to read brand name")?;

    let icon: String = Input::with_theme(&theme)
        .with_prompt("Icon SVG path")
        .default("icon.svg".to_owned())
        .interact_text()
        .context("Failed to read icon path")?;

    let font: String = Input::with_theme(&theme)
        .with_prompt("Font (file path or Google Font name)")
        .default("Inter".to_owned())
        .interact_text()
        .context("Failed to read font")?;

    let config = BrandConfig {
        name: Some(name),
        output_dir: Some(PathBuf::from("brand-output")),
        theme: ThemeConfig {
            font: Some(PathBuf::from(font)),
            ..ThemeConfig::default()
        },
        assets: AssetsConfig {
            icon: IconConfig {
                path: Some(PathBuf::from(icon)),
                ..IconConfig::default()
            },
            ..AssetsConfig::default()
        },
        usvg: UsvgOptions::default(),
    };

    config.save(&args.output)?;
    eprintln!("Wrote {}", args.output.display());

    Ok(())
}
