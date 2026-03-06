# brand

Generate a complete brand icon set from an SVG icon, brand name, and font.

`brand` takes a single SVG icon and a TOML config file and produces a full set of brand assets: favicons, app icons, logos, and wordmarks in both dark and light variants, along with a ready-to-use `site.webmanifest` and HTML `<head>` snippet.

## Installation

```bash
cargo install brand
```

## Quick Start

```bash
# Create a brand.toml interactively
brand init

# Generate assets
brand generate
```

## Configuration

`brand init` creates a `brand.toml` file:

```toml
name = "My Brand"
output-dir = "brand-output"

[theme]
font = "Inter"
font-weight = 700
text-dark = "#fafafa"
text-light = "#18181b"

[assets.icon]
path = "icon.svg"
sizes = [16, 32, 96, 180, 512]

[assets.logo]
wordmark-only = false

[assets.wordmark]
# path = "wordmark.svg"  # optional: use a custom wordmark SVG
```

Fonts can be a local file path or a Google Fonts family name.

## Generated Output

```
brand-output/
  web/
    favicon.ico
    favicon.svg
    favicon-96x96.png
    apple-touch-icon.png
    web-app-manifest-192x192.png
    web-app-manifest-512x512.png
    site.webmanifest
    head.html
  assets/
    icon/
      icon.svg
      icon-{size}x{size}.png
    logo/
      logo-dark.svg, logo-light.svg
      logo-dark-{xs,sm,lg}.png, logo-light-{xs,sm,lg}.png
    wordmark/
      wordmark-dark.svg, wordmark-light.svg
      wordmark-dark-{xs,sm,lg}.png, wordmark-light-{xs,sm,lg}.png
```

All PNGs are optimized with [oxipng](https://github.com/shssoichiro/oxipng). SVG text is converted to paths via [usvg](https://github.com/nicolo-ribaudo/resvg) for maximum compatibility.

## Examples

The [`examples/`](examples/) directory contains sample configurations for well-known brands:

- **[google](examples/google/)** — Wordmark-only logo using a custom SVG wordmark and bundled OpenSans font
- **[microsoft](examples/microsoft/)** — Icon + wordmark logo using a bundled Selawik font with custom theme colors

Run an example:

```bash
cd examples/google
brand generate
```

## Crates

| Crate | Description |
| --- | --- |
| [brand](crates/brand) | CLI tool and asset generation pipeline |
| [usvg-options](crates/usvg-options) | Serializable config struct mirroring `usvg::Options` |

## License

Apache-2.0
