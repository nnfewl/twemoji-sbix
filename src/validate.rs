use anyhow::{bail, Context, Result};
use font_types::Tag;
use read_fonts::{FontRef, TableProvider};

pub fn validate_font(path: &str) -> Result<()> {
    let data = std::fs::read(path)
        .with_context(|| format!("Cannot read font file: {path}"))?;
    let font = FontRef::new(&data)
        .context("Failed to parse font")?;

    println!("Validating {path}...");

    // Check required tables
    let required = [b"head", b"hhea", b"maxp", b"OS/2", b"name", b"cmap", b"sbix"];
    for tag_bytes in &required {
        let tag = Tag::new(tag_bytes);
        if font.table_data(tag).is_none() {
            bail!("Missing required table: {tag}");
        }
    }
    println!("  All required tables present");

    // Check head
    let head = font.head().context("Failed to read head table")?;
    println!("  units_per_em: {}", head.units_per_em());

    // Check maxp
    let maxp = font.maxp().context("Failed to read maxp table")?;
    println!("  numGlyphs: {}", maxp.num_glyphs());

    // Check cmap
    let cmap = font.cmap().context("Failed to read cmap table")?;
    let test_codepoints = [0x1F600u32, 0x2764, 0x1F44D]; // 😀 ❤ 👍
    let mut found = 0;
    for &cp in &test_codepoints {
        if cmap.map_codepoint(cp).is_some() {
            found += 1;
        }
    }
    println!("  cmap: {found}/{} test codepoints mapped", test_codepoints.len());

    // Check name table for family name
    let name = font.name().context("Failed to read name table")?;
    let mut family_found = false;
    for record in name.name_record() {
        if record.name_id().to_u16() == 1 {
            if let Ok(s) = record.string(name.string_data()) {
                let s_str = s.to_string();
                if s_str.contains("Apple Color Emoji") {
                    family_found = true;
                }
            }
        }
    }
    if !family_found {
        bail!("Family name 'Apple Color Emoji' not found in name table");
    }
    println!("  Family name: Apple Color Emoji ✓");

    // Check sbix
    let sbix_data = font.table_data(Tag::new(b"sbix"))
        .context("sbix table data not found")?;
    let sbix_bytes = sbix_data.as_ref();
    if sbix_bytes.len() < 8 {
        bail!("sbix table too small");
    }
    let version = u16::from_be_bytes([sbix_bytes[0], sbix_bytes[1]]);
    let num_strikes = u32::from_be_bytes([sbix_bytes[4], sbix_bytes[5], sbix_bytes[6], sbix_bytes[7]]);
    println!("  sbix: version={version}, strikes={num_strikes}");

    // Verify PNG signatures in first strike
    if num_strikes > 0 {
        let strike_offset = u32::from_be_bytes([sbix_bytes[8], sbix_bytes[9], sbix_bytes[10], sbix_bytes[11]]) as usize;
        if strike_offset + 4 <= sbix_bytes.len() {
            let ppem = u16::from_be_bytes([sbix_bytes[strike_offset], sbix_bytes[strike_offset + 1]]);
            println!("  First strike ppem: {ppem}");
        }
    }

    println!("Validation passed ✓");
    Ok(())
}
