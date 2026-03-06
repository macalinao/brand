extern crate alloc;

use alloc::sync::Arc;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use resvg::tiny_skia::{self, Pixmap};
use resvg::usvg::{self, fontdb};
use serde::Serialize;

use brand_usvg_options::UsvgOptions;

use crate::config::BrandConfig;

/// Shared SVG rendering context (font database + usvg config).
struct SvgContext<'a> {
    fontdb: Arc<fontdb::Database>,
    usvg_opts: &'a UsvgOptions,
}

#[derive(Serialize)]
struct WebManifestIcon {
    src: String,
    sizes: String,
    r#type: String,
    purpose: String,
}

#[derive(Serialize)]
struct WebManifest {
    name: String,
    short_name: String,
    icons: Vec<WebManifestIcon>,
    theme_color: String,
    background_color: String,
    display: String,
}

/// Named PNG size variants: (suffix, height).
/// The default size (96px) uses no suffix.
const PNG_SIZES: &[(&str, u32)] = &[("-xs", 32), ("-sm", 64), ("", 96), ("-lg", 256)];

/// Run the full brand asset generation pipeline.
///
/// Generates favicons, apple touch icons, web manifest, logos, wordmarks, and head.html.
/// All paths in `config` must already be resolved (absolute).
pub fn run(config: &BrandConfig) -> Result<()> {
    let output_dir = config.output_dir()?;

    // Clean and recreate the output directory.
    if output_dir.exists() {
        fs::remove_dir_all(output_dir).context("Failed to clean output directory")?;
    }

    let web_dir = output_dir.join("web");
    let icon_dir = output_dir.join("assets/icon");
    let logo_dir = output_dir.join("assets/logo");
    let wordmark_dir = output_dir.join("assets/wordmark");

    fs::create_dir_all(&web_dir).context("Failed to create web directory")?;
    fs::create_dir_all(&icon_dir).context("Failed to create icon directory")?;
    fs::create_dir_all(&logo_dir).context("Failed to create logo directory")?;
    fs::create_dir_all(&wordmark_dir).context("Failed to create wordmark directory")?;

    let icon_svg = fs::read(config.icon_path()?).context("Failed to read icon SVG")?;
    let font_data = fs::read(config.font()?).context("Failed to read font file")?;

    let mut fontdb = fontdb::Database::new();
    fontdb.load_font_data(font_data);

    let font_family = fontdb
        .faces()
        .next()
        .and_then(|face| face.families.first().map(|(name, _)| name.clone()))
        .context("No font families found in the provided font file")?;

    let ctx = SvgContext {
        fontdb: Arc::new(fontdb),
        usvg_opts: &config.usvg,
    };

    let icon_tree = parse_svg(&icon_svg, &ctx)?;

    eprintln!("Generating brand assets...");
    eprintln!("Font family: {font_family}");
    eprintln!();
    eprintln!("Source: {}", config.icon_path()?.display());
    eprintln!();

    // --- assets/icon/ ---

    eprintln!("assets/icon/");
    fs::copy(config.icon_path()?, icon_dir.join("icon.svg")).context("Failed to copy icon.svg")?;
    eprintln!("  icon.svg");

    for &size in &config.assets.icon.sizes {
        let filename = format!("icon-{size}x{size}.png");
        render_tree_to_png(&icon_tree, size, &icon_dir.join(&filename))?;
        eprintln!("  {filename}");
    }

    generate_web_assets(config, &icon_tree, &web_dir)?;

    let wordmark_svg = config
        .assets
        .wordmark
        .path
        .as_ref()
        .map(|p| {
            fs::read(p).with_context(|| format!("Failed to read wordmark SVG: {}", p.display()))
        })
        .transpose()?;

    let shared = WordmarkAssetsParams {
        config,
        wordmark_svg: wordmark_svg.as_deref(),
        output_dir: &logo_dir,
        ctx: &ctx,
        font_family: &font_family,
    };

    generate_logo_assets(&shared, &icon_svg, &logo_dir)?;
    generate_wordmark_assets(&WordmarkAssetsParams {
        output_dir: &wordmark_dir,
        ..shared
    })?;

    eprintln!();
    eprintln!("Done.");

    Ok(())
}

/// Generate all web/ directory assets (favicons, apple-touch-icon, webmanifest, head.html).
fn generate_web_assets(config: &BrandConfig, icon_tree: &usvg::Tree, web_dir: &Path) -> Result<()> {
    let name = config.name()?;
    eprintln!("web/");

    render_tree_to_png(icon_tree, 96, &web_dir.join("favicon-96x96.png"))?;
    eprintln!("  favicon-96x96.png");

    fs::copy(config.icon_path()?, web_dir.join("favicon.svg"))
        .context("Failed to copy favicon.svg")?;
    eprintln!("  favicon.svg");

    generate_favicon(icon_tree, web_dir)?;
    eprintln!("  favicon.ico (48x48 PNG)");

    render_tree_to_png(icon_tree, 180, &web_dir.join("apple-touch-icon.png"))?;
    eprintln!("  apple-touch-icon.png (180x180)");

    render_tree_to_png(
        icon_tree,
        192,
        &web_dir.join("web-app-manifest-192x192.png"),
    )?;
    eprintln!("  web-app-manifest-192x192.png");

    render_tree_to_png(
        icon_tree,
        512,
        &web_dir.join("web-app-manifest-512x512.png"),
    )?;
    eprintln!("  web-app-manifest-512x512.png");

    generate_webmanifest(name, web_dir)?;
    eprintln!("  site.webmanifest");

    generate_head_html(name, web_dir)?;
    eprintln!("  head.html");

    Ok(())
}

/// Generate logo SVGs and PNGs into the logo output directory.
fn generate_logo_assets(
    params: &WordmarkAssetsParams<'_>,
    icon_svg: &[u8],
    logo_dir: &Path,
) -> Result<()> {
    let WordmarkAssetsParams {
        config,
        wordmark_svg,
        ctx,
        font_family,
        ..
    } = params;

    eprintln!("assets/logo/");

    if config.wordmark_only() {
        generate_wordmark_only_logos(params)?;
    } else {
        for (variant, color) in [
            ("dark", config.text_dark()?),
            ("light", config.text_light()?),
        ] {
            generate_logo_svg(&LogoSvgParams {
                icon_svg,
                wordmark_svg: *wordmark_svg,
                brand_name: config.name()?,
                text_color: color,
                filename: &format!("logo-{variant}.svg"),
                output_dir: logo_dir,
                ctx,
                font_family,
                font_weight: config.font_weight()?,
            })?;
            eprintln!("  logo-{variant}.svg");
        }
    }

    for variant in &["dark", "light"] {
        let svg_data = fs::read(logo_dir.join(format!("logo-{variant}.svg")))?;
        render_svg_pngs_at_sizes(&svg_data, &format!("logo-{variant}"), logo_dir, ctx)?;
    }

    Ok(())
}

#[derive(Clone, Copy)]
struct WordmarkAssetsParams<'a> {
    config: &'a BrandConfig,
    wordmark_svg: Option<&'a [u8]>,
    output_dir: &'a Path,
    ctx: &'a SvgContext<'a>,
    font_family: &'a str,
}

/// Generate wordmark SVGs and PNGs into the wordmark output directory.
fn generate_wordmark_assets(params: &WordmarkAssetsParams<'_>) -> Result<()> {
    let WordmarkAssetsParams {
        config,
        wordmark_svg,
        output_dir,
        ctx,
        font_family,
    } = params;

    eprintln!("assets/wordmark/");

    if let Some(wm) = wordmark_svg {
        fs::write(output_dir.join("wordmark.svg"), wm).context("Failed to write wordmark.svg")?;
        eprintln!("  wordmark.svg");

        render_svg_pngs_at_sizes(wm, "wordmark", output_dir, ctx)?;
    } else {
        let colors = [
            ("dark", config.text_dark()?),
            ("light", config.text_light()?),
        ];
        for (variant, color) in colors {
            let filename = format!("wordmark-{variant}.svg");
            generate_wordmark_svg(&WordmarkSvgParams {
                brand_name: config.name()?,
                text_color: color,
                filename: &filename,
                output_dir,
                ctx,
                font_family,
                font_weight: config.font_weight()?,
            })?;
            eprintln!("  {filename}");

            let svg_data = fs::read(output_dir.join(&filename))?;
            render_svg_pngs_at_sizes(&svg_data, &format!("wordmark-{variant}"), output_dir, ctx)?;
        }
    }

    Ok(())
}

/// Generate logo SVGs that are the wordmark alone (no icon).
fn generate_wordmark_only_logos(params: &WordmarkAssetsParams<'_>) -> Result<()> {
    let WordmarkAssetsParams {
        config,
        wordmark_svg,
        output_dir,
        ctx,
        font_family,
    } = params;

    if let Some(wm) = wordmark_svg {
        for variant in &["dark", "light"] {
            let filename = format!("logo-{variant}.svg");
            fs::write(output_dir.join(&filename), wm)
                .with_context(|| format!("Failed to write {filename}"))?;
            eprintln!("  {filename}");
        }
    } else {
        let colors = [
            ("dark", config.text_dark()?),
            ("light", config.text_light()?),
        ];
        for (variant, color) in colors {
            let filename = format!("logo-{variant}.svg");
            generate_wordmark_svg(&WordmarkSvgParams {
                brand_name: config.name()?,
                text_color: color,
                filename: &filename,
                output_dir,
                ctx,
                font_family,
                font_weight: config.font_weight()?,
            })?;
            eprintln!("  {filename}");
        }
    }
    Ok(())
}

/// Render an SVG to PNGs at each named size in `PNG_SIZES`.
fn render_svg_pngs_at_sizes(
    svg_data: &[u8],
    base_name: &str,
    output_dir: &Path,
    ctx: &SvgContext,
) -> Result<()> {
    for &(suffix, h) in PNG_SIZES {
        let filename = format!("{base_name}{suffix}.png");
        render_svg_to_png_height(svg_data, h, &output_dir.join(&filename), ctx)?;
        eprintln!("  {filename}");
    }
    Ok(())
}

/// Convert u32 to f32 for pixel math.
#[expect(clippy::cast_precision_loss)]
fn to_f32(v: u32) -> f32 {
    v as f32
}

/// Render a parsed SVG tree to a square PNG of the given size.
fn render_tree_to_png(tree: &usvg::Tree, size: u32, output: &Path) -> Result<()> {
    let mut pixmap = Pixmap::new(size, size).context("Failed to create pixmap")?;

    let svg_size = tree.size();
    let size_f = to_f32(size);
    let scale_x = size_f / svg_size.width();
    let scale_y = size_f / svg_size.height();
    let scale = scale_x.min(scale_y);

    let translate_x = (size_f - svg_size.width() * scale) / 2.0;
    let translate_y = (size_f - svg_size.height() * scale) / 2.0;

    let transform =
        tiny_skia::Transform::from_translate(translate_x, translate_y).post_scale(scale, scale);

    resvg::render(tree, transform, &mut pixmap.as_mut());
    save_optimized_png(&pixmap, output)?;

    Ok(())
}

/// Render an SVG to a PNG with a specific height, preserving aspect ratio.
#[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn render_svg_to_png_height(
    svg_data: &[u8],
    height: u32,
    output: &Path,
    ctx: &SvgContext,
) -> Result<()> {
    let tree = parse_svg(svg_data, ctx)?;

    let svg_size = tree.size();
    let scale = to_f32(height) / svg_size.height();
    let width = (svg_size.width() * scale).ceil() as u32;

    let mut pixmap = Pixmap::new(width, height).context("Failed to create pixmap")?;
    let transform = tiny_skia::Transform::from_scale(scale, scale);

    resvg::render(&tree, transform, &mut pixmap.as_mut());
    save_optimized_png(&pixmap, output)?;

    Ok(())
}

/// Encode a pixmap as PNG and optimize it with oxipng before writing to disk.
fn save_optimized_png(pixmap: &Pixmap, output: &Path) -> Result<()> {
    let raw_png = pixmap.encode_png().context("Failed to encode PNG")?;
    let optimized = oxipng::optimize_from_memory(&raw_png, &oxipng::Options::from_preset(2))
        .context("Failed to optimize PNG")?;
    fs::write(output, optimized).context("Failed to write PNG")?;
    Ok(())
}

/// Generate favicon.ico as a 48x48 PNG (served with .ico extension).
fn generate_favicon(tree: &usvg::Tree, output_dir: &Path) -> Result<()> {
    render_tree_to_png(tree, 48, &output_dir.join("favicon.ico"))
}

struct LogoSvgParams<'a> {
    icon_svg: &'a [u8],
    wordmark_svg: Option<&'a [u8]>,
    brand_name: &'a str,
    text_color: &'a str,
    filename: &'a str,
    output_dir: &'a Path,
    ctx: &'a SvgContext<'a>,
    font_family: &'a str,
    font_weight: u32,
}

/// Generate a logo SVG combining the icon with either a wordmark SVG or rendered text.
///
/// When a wordmark SVG is provided it is embedded next to the icon; otherwise the
/// brand name is rendered as a `<text>` element and converted to paths via usvg.
fn generate_logo_svg(params: &LogoSvgParams<'_>) -> Result<()> {
    let LogoSvgParams {
        icon_svg,
        wordmark_svg,
        brand_name,
        text_color,
        filename,
        output_dir,
        ctx,
        font_family,
        font_weight,
    } = params;
    let icon_tree = parse_svg(icon_svg, ctx)?;
    let icon_size = icon_tree.size();
    let icon_h = icon_size.height();
    let icon_w = icon_size.width();

    let icon_svg_str = core::str::from_utf8(icon_svg).context("Icon SVG is not valid UTF-8")?;
    let inner_svg = extract_svg_inner(icon_svg_str);
    let icon_vb = extract_viewbox(icon_svg_str).unwrap_or(format!("0 0 {icon_w} {icon_h}"));

    let gap = icon_h * 0.2;
    let wordmark_x = icon_w + gap;

    let right_half = if let Some(wm) = wordmark_svg {
        let wm_str = core::str::from_utf8(wm).context("Wordmark SVG is not valid UTF-8")?;
        let wm_tree = parse_svg(wm, ctx)?;
        let wm_size = wm_tree.size();
        let wm_vb = extract_viewbox(wm_str).unwrap_or(format!(
            "0 0 {} {}",
            wm_size.width(),
            wm_size.height()
        ));
        let wm_inner = extract_svg_inner(wm_str);
        let wm_scaled_w = wm_size.width() * (icon_h / wm_size.height());
        format!(
            r#"<svg x="{wordmark_x}" viewBox="{wm_vb}" width="{wm_scaled_w}" height="{icon_h}">{wm_inner}</svg>"#,
        )
    } else {
        let font_weight_str = font_weight_to_str(*font_weight);
        let text_y = icon_h * 0.65;
        let font_size = icon_h * 0.45;
        format!(
            r#"<text x="{wordmark_x}" y="{text_y}" font-family="{font_family}" font-weight="{font_weight_str}" font-size="{font_size}" fill="{text_color}" letter-spacing="-0.02em">{brand_name}</text>"#,
        )
    };

    let total_width = icon_w * 4.0;
    let composite_svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {total_width} {icon_h}" width="{total_width}" height="{icon_h}">
  <svg viewBox="{icon_vb}" width="{icon_w}" height="{icon_h}">{inner_svg}</svg>
  {right_half}
</svg>"#,
    );

    let tree = parse_svg(composite_svg.as_bytes(), ctx)?;
    let rendered_size = tree.size();
    let output_svg = tree.to_string(&usvg::WriteOptions::default());
    let cropped = crop_svg_to_content(
        &output_svg,
        rendered_size.width(),
        rendered_size.height(),
        ctx.usvg_opts,
    )?;

    let output_path = output_dir.join(filename);
    fs::write(&output_path, cropped).context("Failed to write logo SVG")?;

    Ok(())
}

struct WordmarkSvgParams<'a> {
    brand_name: &'a str,
    text_color: &'a str,
    filename: &'a str,
    output_dir: &'a Path,
    ctx: &'a SvgContext<'a>,
    font_family: &'a str,
    font_weight: u32,
}

/// Generate a wordmark-only SVG (text rendered to paths, no icon).
fn generate_wordmark_svg(params: &WordmarkSvgParams<'_>) -> Result<()> {
    let WordmarkSvgParams {
        brand_name,
        text_color,
        filename,
        output_dir,
        ctx,
        font_family,
        font_weight,
    } = params;

    let font_weight_str = font_weight_to_str(*font_weight);
    let font_size = 100.0_f32;
    let text_y = font_size * 0.75;
    let canvas_w = font_size * 10.0;
    let canvas_h = font_size * 1.2;

    let text_svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {canvas_w} {canvas_h}" width="{canvas_w}" height="{canvas_h}">
  <text x="0" y="{text_y}" font-family="{font_family}" font-weight="{font_weight_str}" font-size="{font_size}" fill="{text_color}" letter-spacing="-0.02em">{brand_name}</text>
</svg>"#,
    );

    let tree = parse_svg(text_svg.as_bytes(), ctx)?;
    let rendered_size = tree.size();
    let output_svg = tree.to_string(&usvg::WriteOptions::default());
    let cropped = crop_svg_to_content(
        &output_svg,
        rendered_size.width(),
        rendered_size.height(),
        ctx.usvg_opts,
    )?;

    let output_path = output_dir.join(filename);
    fs::write(&output_path, cropped).context("Failed to write wordmark SVG")?;

    Ok(())
}

/// Convert a numeric font weight to a CSS font-weight string.
fn font_weight_to_str(weight: u32) -> &'static str {
    match weight {
        100 => "100",
        200 => "200",
        300 => "300",
        400 => "normal",
        500 => "500",
        600 => "600",
        800 => "800",
        900 => "900",
        _ => "bold",
    }
}

/// Parse SVG data into a usvg tree.
fn parse_svg(data: &[u8], ctx: &SvgContext) -> Result<usvg::Tree> {
    let mut opts = usvg::Options {
        fontdb: Arc::clone(&ctx.fontdb),
        ..usvg::Options::default()
    };
    ctx.usvg_opts.apply_to(&mut opts);

    let tree = usvg::Tree::from_data(data, &opts).context("Failed to parse SVG")?;
    Ok(tree)
}

/// Extract the inner content of an SVG element (everything between `<svg ...>` and `</svg>`).
fn extract_svg_inner(svg: &str) -> &str {
    let svg_open = svg.find("<svg").unwrap_or(0);
    let start = svg[svg_open..].find('>').map_or(0, |i| svg_open + i + 1);
    let end = svg.rfind("</svg>").unwrap_or(svg.len());
    &svg[start..end]
}

/// Extract the viewBox attribute from an SVG string.
fn extract_viewbox(svg: &str) -> Option<String> {
    let vb_start = svg.find("viewBox=\"")?;
    let rest = &svg[vb_start + 9..];
    let vb_end = rest.find('"')?;
    Some(rest[..vb_end].to_string())
}

/// Crop an SVG to its actual content bounding box by rendering and finding bounds.
#[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn crop_svg_to_content(
    svg: &str,
    width: f32,
    height: f32,
    usvg_opts: &UsvgOptions,
) -> Result<String> {
    let w = width.ceil() as u32;
    let h = height.ceil() as u32;
    if w == 0 || h == 0 {
        bail!("SVG has zero dimensions");
    }

    let ctx = SvgContext {
        fontdb: Arc::new(fontdb::Database::new()),
        usvg_opts,
    };
    let tree = parse_svg(svg.as_bytes(), &ctx)?;

    let mut pixmap = Pixmap::new(w, h).context("Failed to create pixmap for bounds detection")?;
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

    let (min_x, min_y, max_x, max_y) = find_content_bounds(&pixmap);

    if min_x > max_x || min_y > max_y {
        return Ok(svg.to_string());
    }

    let padding = 1.0;
    let vb_x = f64::from(min_x) - padding;
    let vb_y = f64::from(min_y) - padding;
    let vb_w = f64::from(max_x - min_x) + padding * 2.0;
    let vb_h = f64::from(max_y - min_y) + padding * 2.0;

    let mut result = svg.to_string();

    if let Some(start) = result.find("viewBox=\"") {
        let rest = &result[start + 9..];
        if let Some(end) = rest.find('"') {
            let old_vb = &result[start..=start + 9 + end];
            let new_vb = format!("viewBox=\"{vb_x} {vb_y} {vb_w} {vb_h}\"");
            result = result.replace(old_vb, &new_vb);
        }
    }

    result = remove_svg_dimension(&result, "width");
    result = remove_svg_dimension(&result, "height");

    Ok(result)
}

/// Remove a dimension attribute (width or height) from the root <svg> element.
fn remove_svg_dimension(svg: &str, attr: &str) -> String {
    let pattern = format!("{attr}=\"");
    if let Some(svg_tag_end) = svg.find('>') {
        let svg_tag = &svg[..svg_tag_end];
        if let Some(attr_start) = svg_tag.find(&pattern) {
            let rest = &svg[attr_start + pattern.len()..];
            if let Some(attr_end) = rest.find('"') {
                let full_attr_end = attr_start + pattern.len() + attr_end + 1;
                let start = if attr_start > 0 && svg.as_bytes()[attr_start - 1] == b' ' {
                    attr_start - 1
                } else {
                    attr_start
                };
                let mut result = String::with_capacity(svg.len());
                result.push_str(&svg[..start]);
                result.push_str(&svg[full_attr_end..]);
                return result;
            }
        }
    }
    svg.to_string()
}

/// Find the bounding box of non-transparent pixels in a pixmap.
fn find_content_bounds(pixmap: &Pixmap) -> (u32, u32, u32, u32) {
    let w = pixmap.width();
    let h = pixmap.height();
    let data = pixmap.data();

    let mut min_x = w;
    let mut min_y = h;
    let mut max_x = 0_u32;
    let mut max_y = 0_u32;

    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize * 4;
            let alpha = data[idx + 3];
            if alpha > 0 {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
    }

    (min_x, min_y, max_x, max_y)
}

/// Generate site.webmanifest with PWA icon entries.
fn generate_webmanifest(name: &str, output_dir: &Path) -> Result<()> {
    let manifest = WebManifest {
        name: name.to_string(),
        short_name: name.to_string(),
        icons: vec![
            WebManifestIcon {
                src: "/web-app-manifest-192x192.png".to_string(),
                sizes: "192x192".to_string(),
                r#type: "image/png".to_string(),
                purpose: "maskable".to_string(),
            },
            WebManifestIcon {
                src: "/web-app-manifest-512x512.png".to_string(),
                sizes: "512x512".to_string(),
                r#type: "image/png".to_string(),
                purpose: "maskable".to_string(),
            },
        ],
        theme_color: "#ffffff".to_string(),
        background_color: "#ffffff".to_string(),
        display: "standalone".to_string(),
    };

    let json = serde_json::to_string_pretty(&manifest).context("Failed to serialize manifest")?;

    let output_path = output_dir.join("site.webmanifest");
    fs::write(&output_path, json).context("Failed to write site.webmanifest")?;

    Ok(())
}

/// Generate head.html with the favicon/meta HTML tags for pasting into `<head>`.
fn generate_head_html(name: &str, output_dir: &Path) -> Result<()> {
    let html = format!(
        r#"<link rel="icon" type="image/png" href="/favicon-96x96.png" sizes="96x96" />
<link rel="icon" type="image/svg+xml" href="/favicon.svg" />
<link rel="shortcut icon" href="/favicon.ico" />
<link rel="apple-touch-icon" sizes="180x180" href="/apple-touch-icon.png" />
<meta name="apple-mobile-web-app-title" content="{name}" />
<link rel="manifest" href="/site.webmanifest" />
"#
    );

    let output_path = output_dir.join("head.html");
    fs::write(&output_path, html).context("Failed to write head.html")?;

    Ok(())
}
