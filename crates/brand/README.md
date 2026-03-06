# brand

Generate a complete brand icon set from an SVG icon, brand name, and font.

## Usage

```bash
# Create a brand.toml interactively
brand init

# Generate assets from brand.toml
brand generate

# Generate from a custom config path
brand generate --config my-brand.toml
```

## Commands

### `brand init`

Interactively creates a `brand.toml` configuration file. Prompts for:

- Brand name
- Path to source SVG icon
- Font (local file or Google Fonts family name)

### `brand generate`

Reads `brand.toml` and generates the full set of brand assets:

- **Favicons**: `favicon.ico`, `favicon.svg`, `favicon-96x96.png`
- **App icons**: Apple touch icon, web app manifest icons at 192px and 512px
- **Icons**: Source SVG + PNGs at configurable sizes (default: 16, 32, 96, 180, 512)
- **Logos**: Dark and light SVG variants + PNG exports (xs, sm, lg)
- **Wordmarks**: Dark and light SVG variants + PNG exports (xs, sm, lg)
- **Web**: `site.webmanifest` and `head.html` snippet

All CLI options from the config can be overridden via command-line flags.

## Configuration

See the [root README](../../README.md#configuration) for the full `brand.toml` schema.

## License

Apache-2.0
