mod config;
mod font;
mod rasterize;
mod validate;

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use config::{get_preset, list_presets};

#[derive(Parser)]
#[command(name = "twemoji-sbix", about = "Build Twemoji sbix font from SVGs")]
struct Cli {
    /// Strike size preset
    #[arg(long, default_value = "optimal")]
    preset: String,

    /// Custom strike sizes (overrides --preset)
    #[arg(long, num_args = 1..)]
    sizes: Option<Vec<u16>>,

    /// Output file path
    #[arg(short, long, default_value = "Twemoji.ttc")]
    output: String,

    /// Path to Twemoji SVG directory
    #[arg(long, default_value = "twemoji/assets/svg")]
    svg_dir: PathBuf,

    /// Show available presets and exit
    #[arg(long)]
    list_presets: bool,

    /// Skip validation after building
    #[arg(long)]
    no_validate: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.list_presets {
        println!("Available presets:\n");
        println!("{}", list_presets());
        return Ok(());
    }

    let sizes = if let Some(ref custom) = cli.sizes {
        custom.clone()
    } else {
        let preset = get_preset(&cli.preset)
            .ok_or_else(|| anyhow::anyhow!("Unknown preset '{}'. Use --list-presets to see options.", cli.preset))?;
        preset.sizes.to_vec()
    };

    println!("Preset: {} → strikes {:?}", cli.preset, sizes);

    // Rasterize
    let glyphs = rasterize::rasterize_all(&cli.svg_dir, &sizes)?;

    // Plan glyph order
    let font_data = font::plan_glyphs(&glyphs);

    // Build font
    font::build_font(&glyphs, &font_data, &sizes, &cli.output)?;

    // Validate
    if !cli.no_validate {
        println!();
        validate::validate_font(&cli.output)?;
    }

    Ok(())
}
