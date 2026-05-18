use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use rayon::prelude::*;
use resvg::tiny_skia::Pixmap;
use resvg::usvg;

pub struct RasterizedGlyph {
    pub stem: String,
    pub codepoints: Vec<u32>,
    pub glyph_name: String,
    pub png_data: BTreeMap<u16, Vec<u8>>,
}

fn codepoints_from_stem(stem: &str) -> Vec<u32> {
    stem.split('-')
        .map(|s| u32::from_str_radix(s, 16).unwrap())
        .collect()
}

fn glyph_name_from_stem(stem: &str) -> String {
    let parts: Vec<String> = stem.split('-').map(|s| s.to_uppercase()).collect();
    format!("u{}", parts.join("_"))
}

fn encode_png(pixmap: &Pixmap) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    let mut encoder = png::Encoder::new(&mut buf, pixmap.width(), pixmap.height());
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(pixmap.data())?;
    writer.finish()?;
    Ok(buf)
}

pub fn rasterize_all(svg_dir: &Path, sizes: &[u16]) -> Result<Vec<RasterizedGlyph>> {
    let mut svg_entries: Vec<(String, Vec<u8>)> = Vec::new();

    let entries: Vec<_> = fs::read_dir(svg_dir)
        .with_context(|| format!("Cannot read SVG directory: {}", svg_dir.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "svg"))
        .collect();

    for entry in &entries {
        let path = entry.path();
        let stem = path.file_stem().unwrap().to_string_lossy().to_string();
        let data = fs::read(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        svg_entries.push((stem, data));
    }

    svg_entries.sort_by(|a, b| a.0.cmp(&b.0));
    println!("Found {} SVGs", svg_entries.len());
    println!("Rasterizing at sizes: {:?}", sizes);

    let results: Vec<Result<RasterizedGlyph>> = svg_entries
        .par_iter()
        .map(|(stem, svg_data)| {
            let tree = usvg::Tree::from_data(svg_data, &usvg::Options::default())
                .with_context(|| format!("Failed to parse SVG: {stem}"))?;

            let codepoints = codepoints_from_stem(stem);
            let glyph_name = glyph_name_from_stem(stem);
            let mut png_data = BTreeMap::new();

            for &size in sizes {
                let mut pixmap = Pixmap::new(size as u32, size as u32)
                    .context("Failed to create pixmap")?;

                let sx = size as f32 / tree.size().width();
                let sy = size as f32 / tree.size().height();
                let transform = resvg::tiny_skia::Transform::from_scale(sx, sy);

                resvg::render(&tree, transform, &mut pixmap.as_mut());
                let png_bytes = encode_png(&pixmap)?;
                png_data.insert(size, png_bytes);
            }

            Ok(RasterizedGlyph {
                stem: stem.clone(),
                codepoints,
                glyph_name,
                png_data,
            })
        })
        .collect();

    let mut glyphs = Vec::with_capacity(results.len());
    let mut errors = 0;
    for result in results {
        match result {
            Ok(g) => glyphs.push(g),
            Err(e) => {
                if errors < 5 {
                    eprintln!("  Error: {e}");
                }
                errors += 1;
            }
        }
    }

    if errors > 0 {
        eprintln!("{errors} SVGs failed to rasterize");
    }
    println!("Rasterized {} glyphs", glyphs.len());
    Ok(glyphs)
}
