# usvg-options

Serializable config struct mirroring the configurable subset of [`usvg::Options`](https://docs.rs/usvg/latest/usvg/struct.Options.html).

## Purpose

`usvg::Options` doesn't implement `Serialize`/`Deserialize`. This crate provides `UsvgOptions`, a mirror struct that derives `Serialize`, `Deserialize`, `JsonSchema`, and `Bpaf` (for CLI argument parsing), making it easy to embed SVG rendering options in config files and CLI tools.

## Usage

```rust
use usvg_options::UsvgOptions;

// Deserialize from TOML/JSON config
let config: UsvgOptions = toml::from_str(r#"
    dpi = 96.0
    font-family = "Arial"
"#)?;

// Apply to usvg::Options
let mut opts = usvg::Options::default();
config.apply_to(&mut opts);
```

## Supported Fields

| Field | Type | Default |
| --- | --- | --- |
| `dpi` | `f32` | `96.0` |
| `font-family` | `String` | `"Times New Roman"` |
| `font-size` | `f32` | `12.0` |
| `languages` | `Vec<String>` | `["en"]` |
| `shape-rendering` | `enum` | `geometric-precision` |
| `text-rendering` | `enum` | `optimize-legibility` |
| `image-rendering` | `enum` | `optimize-quality` |
| `default-size` | `(f32, f32)` | `(100.0, 100.0)` |
| `style-sheet` | `String?` | `None` |

## License

Apache-2.0
