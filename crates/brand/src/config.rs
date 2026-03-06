use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bpaf::{Bpaf, ShellComp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use brand_usvg_options::{UsvgOptions, usvg_options};

use crate::fonts;

#[allow(clippy::unnecessary_wraps)]
fn default_font() -> Option<PathBuf> {
    Some(PathBuf::from("Inter"))
}

#[allow(clippy::unnecessary_wraps)]
fn default_font_weight() -> Option<u32> {
    Some(700)
}

#[allow(clippy::unnecessary_wraps)]
fn default_output_dir() -> Option<PathBuf> {
    Some(PathBuf::from("brand-output"))
}

#[allow(clippy::unnecessary_wraps)]
fn default_text_dark() -> Option<String> {
    Some("#fafafa".to_owned())
}

#[allow(clippy::unnecessary_wraps)]
fn default_text_light() -> Option<String> {
    Some("#18181b".to_owned())
}

fn default_icon_sizes() -> Vec<u32> {
    vec![16, 32, 96, 180, 512]
}

#[allow(clippy::unnecessary_wraps)]
fn default_icon_path() -> Option<PathBuf> {
    Some(PathBuf::from("icon.svg"))
}

#[allow(clippy::unnecessary_wraps)]
fn default_wordmark_only() -> Option<bool> {
    Some(false)
}

/// Theme overrides
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Bpaf)]
pub struct ThemeConfig {
    /// Font file path or Google Fonts family name (e.g. "Inter").
    #[serde(default = "default_font", rename = "font")]
    #[bpaf(long)]
    pub font: Option<PathBuf>,
    /// Font weight.
    #[serde(default = "default_font_weight", rename = "font-weight")]
    #[bpaf(long)]
    pub font_weight: Option<u32>,
    /// Text color for the dark variant (light text on dark background).
    #[serde(default = "default_text_dark", rename = "text-dark")]
    #[bpaf(long)]
    pub text_dark: Option<String>,
    /// Text color for the light variant (dark text on light background).
    #[serde(default = "default_text_light", rename = "text-light")]
    #[bpaf(long)]
    pub text_light: Option<String>,
}

#[allow(clippy::derivable_impls)]
impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            font: default_font(),
            font_weight: default_font_weight(),
            text_dark: default_text_dark(),
            text_light: default_text_light(),
        }
    }
}

/// Asset overrides
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, Bpaf)]
pub struct AssetsConfig {
    /// `[assets.icon]` — icon source configuration.
    #[serde(default)]
    #[bpaf(external(icon_config))]
    pub icon: IconConfig,
    /// `[assets.logo]` — logo generation options.
    #[serde(default)]
    #[bpaf(external(logo_config))]
    pub logo: LogoConfig,
    /// `[assets.wordmark]` — wordmark generation options.
    #[serde(default)]
    #[bpaf(external(wordmark_config))]
    pub wordmark: WordmarkConfig,
}

/// `[assets.icon]` options.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Bpaf)]
pub struct IconConfig {
    /// Path to the source SVG icon (relative to `brand.toml`).
    #[serde(default = "default_icon_path")]
    #[bpaf(long("icon-path"))]
    pub path: Option<PathBuf>,
    /// Icon sizes to generate.
    #[serde(default = "default_icon_sizes")]
    #[bpaf(pure(default_icon_sizes()))]
    pub sizes: Vec<u32>,
}

impl Default for IconConfig {
    fn default() -> Self {
        Self {
            path: default_icon_path(),
            sizes: default_icon_sizes(),
        }
    }
}

/// `[assets.logo]` options.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Bpaf)]
pub struct LogoConfig {
    /// When true, the logo is the wordmark alone (no icon).
    #[serde(default = "default_wordmark_only", rename = "wordmark-only")]
    #[bpaf(long("wordmark-only"), argument("true|false"))]
    pub wordmark_only: Option<bool>,
}

impl Default for LogoConfig {
    fn default() -> Self {
        Self {
            wordmark_only: default_wordmark_only(),
        }
    }
}

/// `[assets.wordmark]` options.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, Bpaf)]
pub struct WordmarkConfig {
    /// Path to an SVG wordmark (relative to `brand.toml`).
    /// When set, logo variants use this SVG instead of rendering text from the font.
    #[bpaf(long("wordmark-path"))]
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Bpaf)]
pub struct BrandConfig {
    /// Brand name to render in the logo.
    #[bpaf(long)]
    pub name: Option<String>,
    /// Output directory (relative to `brand.toml`).
    #[serde(default = "default_output_dir", rename = "output-dir")]
    #[bpaf(short, long, complete_shell(ShellComp::Dir { mask: None }))]
    pub output_dir: Option<PathBuf>,
    /// Font and color theming.
    #[serde(default)]
    #[bpaf(external(theme_config))]
    pub theme: ThemeConfig,
    /// Asset generation options.
    #[serde(default)]
    #[bpaf(external(assets_config))]
    pub assets: AssetsConfig,
    /// usvg rendering options.
    #[serde(default)]
    #[bpaf(external(usvg_options))]
    pub usvg: UsvgOptions,
}

impl BrandConfig {
    /// Load a `BrandConfig` from a TOML file.
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        toml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))
    }

    /// Save this config to a TOML file.
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self).context("Failed to serialize brand config")?;
        std::fs::write(path, content).with_context(|| format!("Failed to write {}", path.display()))
    }

    /// Merge CLI overrides into this config. `Some` values from `overrides`
    /// replace the corresponding field; `None` values are ignored.
    pub fn merge(&mut self, overrides: Self) {
        if overrides.name.is_some() {
            self.name = overrides.name;
        }
        if overrides.output_dir.is_some() {
            self.output_dir = overrides.output_dir;
        }
        // Theme
        if overrides.theme.font.is_some() {
            self.theme.font = overrides.theme.font;
        }
        if overrides.theme.font_weight.is_some() {
            self.theme.font_weight = overrides.theme.font_weight;
        }
        if overrides.theme.text_dark.is_some() {
            self.theme.text_dark = overrides.theme.text_dark;
        }
        if overrides.theme.text_light.is_some() {
            self.theme.text_light = overrides.theme.text_light;
        }
        // Assets
        if overrides.assets.icon.path.is_some() {
            self.assets.icon.path = overrides.assets.icon.path;
        }
        if overrides.assets.logo.wordmark_only.is_some() {
            self.assets.logo.wordmark_only = overrides.assets.logo.wordmark_only;
        }
        if overrides.assets.wordmark.path.is_some() {
            self.assets.wordmark.path = overrides.assets.wordmark.path;
        }
        // Usvg
        self.usvg.merge(&overrides.usvg);
    }

    /// Resolve all relative paths against `base_dir` so they become absolute.
    ///
    /// Font is resolved via Google Fonts download if it does not look like a file path.
    pub fn resolve_paths(&mut self, base_dir: &Path) -> Result<()> {
        let font = self.font()?;
        let font_weight = self.font_weight()?;
        self.theme.font = Some(fonts::resolve_font(font, base_dir, font_weight)?);
        self.assets.icon.path = Some(base_dir.join(self.icon_path()?));
        self.output_dir = Some(base_dir.join(self.output_dir()?));
        if let Some(p) = &self.assets.wordmark.path {
            self.assets.wordmark.path = Some(base_dir.join(p));
        }
        Ok(())
    }

    // --- accessors that unwrap required/defaulted fields ---

    pub fn name(&self) -> Result<&str> {
        self.name.as_deref().context("Missing required field: name")
    }

    pub fn output_dir(&self) -> Result<&Path> {
        self.output_dir
            .as_deref()
            .context("Missing required field: output-dir")
    }

    pub fn font(&self) -> Result<&Path> {
        self.theme
            .font
            .as_deref()
            .context("Missing required field: theme.font")
    }

    pub fn font_weight(&self) -> Result<u32> {
        self.theme
            .font_weight
            .context("Missing required field: theme.font-weight")
    }

    pub fn text_dark(&self) -> Result<&str> {
        self.theme
            .text_dark
            .as_deref()
            .context("Missing required field: theme.text-dark")
    }

    pub fn text_light(&self) -> Result<&str> {
        self.theme
            .text_light
            .as_deref()
            .context("Missing required field: theme.text-light")
    }

    pub fn icon_path(&self) -> Result<&Path> {
        self.assets
            .icon
            .path
            .as_deref()
            .context("Missing required field: assets.icon.path")
    }

    pub fn wordmark_only(&self) -> bool {
        self.assets.logo.wordmark_only.unwrap_or(false)
    }
}
