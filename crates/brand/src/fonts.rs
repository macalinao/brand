use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

/// Resolve a font specifier to a file path.
///
/// If the input looks like a file path (contains `.` or `/`), it is resolved
/// relative to `base_dir`. Otherwise, it is treated as a Google Fonts family
/// name and downloaded.
pub fn resolve_font(input: &Path, base_dir: &Path, weight: u32) -> Result<PathBuf> {
    let s = input.to_string_lossy();
    if s.contains('.') || s.contains('/') {
        return Ok(base_dir.join(input));
    }

    download_google_font(&s, weight, base_dir)
}

/// Download a font from Google Fonts CSS2 API.
///
/// Queries with a browser-like User-Agent to get direct TTF URLs.
/// Saves the font file into `<base_dir>/assets/fonts/`.
fn download_google_font(family: &str, weight: u32, base_dir: &Path) -> Result<PathBuf> {
    let url = format!(
        "https://fonts.googleapis.com/css2?family={}:wght@{weight}",
        family.replace(' ', "+")
    );

    eprintln!("Fetching font from Google Fonts...");

    let css: String = ureq::get(&url)
        .header(
            "User-Agent",
            "Mozilla/4.0 (compatible; MSIE 6.0; Windows NT 5.1)",
        )
        .call()
        .context("Failed to fetch Google Fonts CSS")?
        .body_mut()
        .read_to_string()
        .context("Failed to read Google Fonts CSS response")?;

    let ttf_url =
        extract_ttf_url(&css).context("Could not find TTF URL in Google Fonts CSS response")?;

    let font_bytes: Vec<u8> = ureq::get(ttf_url)
        .call()
        .context("Failed to download font file")?
        .body_mut()
        .read_to_vec()
        .context("Failed to read font bytes")?;

    if font_bytes.is_empty() {
        bail!("Downloaded font file is empty");
    }

    let dir = base_dir.join("assets/fonts");
    fs::create_dir_all(&dir).context("Failed to create assets/fonts directory")?;

    let filename = format!("{}-{weight}.ttf", family.replace(' ', ""));
    let path = dir.join(&filename);
    fs::write(&path, &font_bytes)
        .with_context(|| format!("Failed to write font to {}", path.display()))?;

    eprintln!("Downloaded {filename} to {}", path.display());

    Ok(path)
}

/// Extract the first TTF URL from a Google Fonts CSS response.
fn extract_ttf_url(css: &str) -> Option<&str> {
    let marker = "url(";
    let start = css.find(marker)? + marker.len();
    let rest = &css[start..];
    let end = rest.find(')')?;
    let url = rest[..end].trim();
    if url.contains(".ttf") {
        Some(url)
    } else {
        None
    }
}
