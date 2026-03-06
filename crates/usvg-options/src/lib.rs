use core::str::FromStr;

use bpaf::Bpaf;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// SVG rendering options
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Bpaf)]
#[serde(default, rename_all = "kebab-case")]
pub struct UsvgOptions {
    /// Target DPI (impacts units conversion).
    #[bpaf(long)]
    pub dpi: Option<f32>,
    /// Default font family when none is set in the SVG.
    #[bpaf(long("usvg-font-family"))]
    pub font_family: Option<String>,
    /// Default font size when none is set in the SVG.
    #[bpaf(long("usvg-font-size"))]
    pub font_size: Option<f32>,
    /// Languages for `systemLanguage` resolution.
    #[bpaf(pure(default_languages()))]
    pub languages: Vec<String>,
    /// Default shape rendering method.
    #[bpaf(long)]
    pub shape_rendering: Option<ShapeRenderingOpt>,
    /// Default text rendering method.
    #[bpaf(long)]
    pub text_rendering: Option<TextRenderingOpt>,
    /// Default image rendering method.
    #[bpaf(long)]
    pub image_rendering: Option<ImageRenderingOpt>,
    /// Default viewport size `(width, height)` when the SVG lacks a `viewBox`.
    #[bpaf(pure(default_size()))]
    pub default_size: (f32, f32),
    /// Optional CSS stylesheet to inject into the SVG.
    #[bpaf(long)]
    pub style_sheet: Option<String>,
}

fn default_languages() -> Vec<String> {
    usvg::Options::default().languages
}

fn default_size() -> (f32, f32) {
    let s = usvg::Options::default().default_size;
    (s.width(), s.height())
}

impl Default for UsvgOptions {
    fn default() -> Self {
        let opts = usvg::Options::default();
        Self {
            dpi: Some(opts.dpi),
            font_family: Some(opts.font_family),
            font_size: Some(opts.font_size),
            languages: opts.languages,
            shape_rendering: Some(opts.shape_rendering.into()),
            text_rendering: Some(opts.text_rendering.into()),
            image_rendering: Some(opts.image_rendering.into()),
            default_size: (opts.default_size.width(), opts.default_size.height()),
            style_sheet: opts.style_sheet,
        }
    }
}

impl UsvgOptions {
    /// Apply these options onto a mutable `usvg::Options`.
    pub fn apply_to(&self, opts: &mut usvg::Options) {
        if let Some(dpi) = self.dpi {
            opts.dpi = dpi;
        }
        if let Some(ref family) = self.font_family {
            opts.font_family.clone_from(family);
        }
        if let Some(size) = self.font_size {
            opts.font_size = size;
        }
        opts.languages.clone_from(&self.languages);
        if let Some(sr) = self.shape_rendering {
            opts.shape_rendering = sr.into();
        }
        if let Some(tr) = self.text_rendering {
            opts.text_rendering = tr.into();
        }
        if let Some(ir) = self.image_rendering {
            opts.image_rendering = ir.into();
        }
        if let Some(size) = usvg::Size::from_wh(self.default_size.0, self.default_size.1) {
            opts.default_size = size;
        }
        opts.style_sheet.clone_from(&self.style_sheet);
    }

    /// Merge CLI overrides into this config.
    pub fn merge(&mut self, overrides: &Self) {
        if overrides.dpi.is_some() {
            self.dpi = overrides.dpi;
        }
        if overrides.font_family.is_some() {
            self.font_family.clone_from(&overrides.font_family);
        }
        if overrides.font_size.is_some() {
            self.font_size = overrides.font_size;
        }
        if overrides.shape_rendering.is_some() {
            self.shape_rendering = overrides.shape_rendering;
        }
        if overrides.text_rendering.is_some() {
            self.text_rendering = overrides.text_rendering;
        }
        if overrides.image_rendering.is_some() {
            self.image_rendering = overrides.image_rendering;
        }
        if overrides.style_sheet.is_some() {
            self.style_sheet.clone_from(&overrides.style_sheet);
        }
    }
}

// --- Rendering enums ---

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ShapeRenderingOpt {
    OptimizeSpeed,
    CrispEdges,
    #[default]
    GeometricPrecision,
}

impl FromStr for ShapeRenderingOpt {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "optimize-speed" => Ok(Self::OptimizeSpeed),
            "crisp-edges" => Ok(Self::CrispEdges),
            "geometric-precision" => Ok(Self::GeometricPrecision),
            _ => Err(format!("invalid shape-rendering: {s}")),
        }
    }
}

impl From<usvg::ShapeRendering> for ShapeRenderingOpt {
    fn from(val: usvg::ShapeRendering) -> Self {
        match val {
            usvg::ShapeRendering::OptimizeSpeed => Self::OptimizeSpeed,
            usvg::ShapeRendering::CrispEdges => Self::CrispEdges,
            usvg::ShapeRendering::GeometricPrecision => Self::GeometricPrecision,
        }
    }
}

impl From<ShapeRenderingOpt> for usvg::ShapeRendering {
    fn from(val: ShapeRenderingOpt) -> Self {
        match val {
            ShapeRenderingOpt::OptimizeSpeed => Self::OptimizeSpeed,
            ShapeRenderingOpt::CrispEdges => Self::CrispEdges,
            ShapeRenderingOpt::GeometricPrecision => Self::GeometricPrecision,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "kebab-case")]
pub enum TextRenderingOpt {
    OptimizeSpeed,
    #[default]
    OptimizeLegibility,
    GeometricPrecision,
}

impl FromStr for TextRenderingOpt {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "optimize-speed" => Ok(Self::OptimizeSpeed),
            "optimize-legibility" => Ok(Self::OptimizeLegibility),
            "geometric-precision" => Ok(Self::GeometricPrecision),
            _ => Err(format!("invalid text-rendering: {s}")),
        }
    }
}

impl From<usvg::TextRendering> for TextRenderingOpt {
    fn from(val: usvg::TextRendering) -> Self {
        match val {
            usvg::TextRendering::OptimizeSpeed => Self::OptimizeSpeed,
            usvg::TextRendering::OptimizeLegibility => Self::OptimizeLegibility,
            usvg::TextRendering::GeometricPrecision => Self::GeometricPrecision,
        }
    }
}

impl From<TextRenderingOpt> for usvg::TextRendering {
    fn from(val: TextRenderingOpt) -> Self {
        match val {
            TextRenderingOpt::OptimizeSpeed => Self::OptimizeSpeed,
            TextRenderingOpt::OptimizeLegibility => Self::OptimizeLegibility,
            TextRenderingOpt::GeometricPrecision => Self::GeometricPrecision,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ImageRenderingOpt {
    #[default]
    OptimizeQuality,
    OptimizeSpeed,
    Smooth,
    HighQuality,
    CrispEdges,
    Pixelated,
}

impl FromStr for ImageRenderingOpt {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "optimize-quality" => Ok(Self::OptimizeQuality),
            "optimize-speed" => Ok(Self::OptimizeSpeed),
            "smooth" => Ok(Self::Smooth),
            "high-quality" => Ok(Self::HighQuality),
            "crisp-edges" => Ok(Self::CrispEdges),
            "pixelated" => Ok(Self::Pixelated),
            _ => Err(format!("invalid image-rendering: {s}")),
        }
    }
}

impl From<usvg::ImageRendering> for ImageRenderingOpt {
    fn from(val: usvg::ImageRendering) -> Self {
        match val {
            usvg::ImageRendering::OptimizeQuality => Self::OptimizeQuality,
            usvg::ImageRendering::OptimizeSpeed => Self::OptimizeSpeed,
            usvg::ImageRendering::Smooth => Self::Smooth,
            usvg::ImageRendering::HighQuality => Self::HighQuality,
            usvg::ImageRendering::CrispEdges => Self::CrispEdges,
            usvg::ImageRendering::Pixelated => Self::Pixelated,
        }
    }
}

impl From<ImageRenderingOpt> for usvg::ImageRendering {
    fn from(val: ImageRenderingOpt) -> Self {
        match val {
            ImageRenderingOpt::OptimizeQuality => Self::OptimizeQuality,
            ImageRenderingOpt::OptimizeSpeed => Self::OptimizeSpeed,
            ImageRenderingOpt::Smooth => Self::Smooth,
            ImageRenderingOpt::HighQuality => Self::HighQuality,
            ImageRenderingOpt::CrispEdges => Self::CrispEdges,
            ImageRenderingOpt::Pixelated => Self::Pixelated,
        }
    }
}
