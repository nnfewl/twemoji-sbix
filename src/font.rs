use std::collections::BTreeMap;

use anyhow::{Context, Result};
use font_types::{FWord, Fixed, LongDateTime, NameId, Tag, UfWord};
use write_fonts::FontBuilder;
use write_fonts::tables::{
    cmap::{Cmap, CmapSubtable, EncodingRecord, PlatformId, SequentialMapGroup},
    head::{Flags, Head, MacStyle},
    hhea::Hhea,
    hmtx::{Hmtx, LongMetric},
    maxp::Maxp,
    name::{Name, NameRecord},
    os2::Os2,
    post::Post,
};

use crate::config::{ADVANCE_WIDTH, ASCENT, DESCENT, FAMILY_NAME, UNITS_PER_EM};
use crate::rasterize::RasterizedGlyph;

pub struct FontData {
    pub glyph_order: Vec<String>,
    pub single_cp: Vec<(u32, String)>,
    pub multi_cp: Vec<String>,
}

pub fn plan_glyphs(glyphs: &[RasterizedGlyph]) -> FontData {
    let mut single_cp: Vec<(u32, String)> = Vec::new();
    let mut multi_cp: Vec<String> = Vec::new();

    for g in glyphs {
        if g.codepoints.len() == 1 {
            single_cp.push((g.codepoints[0], g.glyph_name.clone()));
        } else {
            multi_cp.push(g.glyph_name.clone());
        }
    }

    single_cp.sort_by_key(|(cp, _)| *cp);
    multi_cp.sort();

    let mut glyph_order = Vec::with_capacity(1 + single_cp.len() + multi_cp.len());
    glyph_order.push(".notdef".to_string());
    for (_, name) in &single_cp {
        glyph_order.push(name.clone());
    }
    for name in &multi_cp {
        glyph_order.push(name.clone());
    }

    println!("Total glyphs: {}", glyph_order.len());
    println!("  Single-codepoint (in cmap): {}", single_cp.len());
    println!("  Multi-codepoint (need morx/GSUB): {}", multi_cp.len());

    FontData { glyph_order, single_cp, multi_cp }
}

fn build_sbix_bytes(
    glyphs: &[RasterizedGlyph],
    glyph_order: &[String],
    sizes: &[u16],
) -> Vec<u8> {
    let glyph_png_map: BTreeMap<&str, &BTreeMap<u16, Vec<u8>>> = glyphs
        .iter()
        .map(|g| (g.glyph_name.as_str(), &g.png_data))
        .collect();

    let num_glyphs = glyph_order.len() as u32;
    let num_strikes = sizes.len() as u32;

    struct StrikeData {
        data: Vec<u8>,
    }

    let mut strikes: Vec<StrikeData> = Vec::new();

    for &ppem in sizes {
        let offsets_size = (num_glyphs + 1) as usize * 4;
        let header_size = 4 + offsets_size;

        let mut glyph_records: Vec<Vec<u8>> = Vec::new();
        for name in glyph_order {
            if let Some(png_map) = glyph_png_map.get(name.as_str()) {
                if let Some(png_bytes) = png_map.get(&ppem) {
                    let mut record = Vec::with_capacity(8 + png_bytes.len());
                    record.extend_from_slice(&0i16.to_be_bytes());
                    record.extend_from_slice(&0i16.to_be_bytes());
                    record.extend_from_slice(b"png ");
                    record.extend_from_slice(png_bytes);
                    glyph_records.push(record);
                    continue;
                }
            }
            glyph_records.push(Vec::new());
        }

        let mut offsets: Vec<u32> = Vec::with_capacity(num_glyphs as usize + 1);
        let mut current_offset = header_size as u32;
        for record in &glyph_records {
            offsets.push(current_offset);
            current_offset += record.len() as u32;
        }
        offsets.push(current_offset);

        let mut strike_bytes = Vec::new();
        strike_bytes.extend_from_slice(&ppem.to_be_bytes());
        strike_bytes.extend_from_slice(&72u16.to_be_bytes());
        for offset in &offsets {
            strike_bytes.extend_from_slice(&offset.to_be_bytes());
        }
        for record in &glyph_records {
            strike_bytes.extend_from_slice(record);
        }

        strikes.push(StrikeData { data: strike_bytes });
    }

    let sbix_header_size = 4 + 4 + num_strikes as usize * 4;

    let mut strike_offsets: Vec<u32> = Vec::new();
    let mut offset = sbix_header_size as u32;
    for strike in &strikes {
        strike_offsets.push(offset);
        offset += strike.data.len() as u32;
    }

    let mut sbix = Vec::new();
    sbix.extend_from_slice(&1u16.to_be_bytes());
    sbix.extend_from_slice(&1u16.to_be_bytes());
    sbix.extend_from_slice(&num_strikes.to_be_bytes());
    for so in &strike_offsets {
        sbix.extend_from_slice(&so.to_be_bytes());
    }
    for strike in &strikes {
        sbix.extend_from_slice(&strike.data);
    }

    sbix
}

pub fn build_font(
    glyphs: &[RasterizedGlyph],
    font_data: &FontData,
    sizes: &[u16],
    output: &str,
) -> Result<()> {
    let num_glyphs = font_data.glyph_order.len() as u16;

    println!("Building sbix table ({} strikes)...", sizes.len());
    let sbix_bytes = build_sbix_bytes(glyphs, &font_data.glyph_order, sizes);

    let mut builder = FontBuilder::default();

    // head
    let flags = Flags::BASELINE_AT_Y_0 | Flags::LSB_AT_X_0 | Flags::FORCE_INTEGER_PPEM;
    let head = Head::new(
        Fixed::from_f64(1.0),   // font_revision
        0,                      // checksum_adjustment (recalculated by consumers)
        flags,
        UNITS_PER_EM,
        LongDateTime::new(0),
        LongDateTime::new(0),
        0, 0, 0, 0,            // x/y min/max bounding box
        MacStyle::empty(),
        8,                      // lowest_rec_ppem
        1,                      // index_to_loc_format (long)
    );
    builder.add_table(&head)?;

    // hhea
    let hhea = Hhea {
        ascender: FWord::new(ASCENT),
        descender: FWord::new(-DESCENT),
        line_gap: FWord::new(0),
        number_of_h_metrics: num_glyphs,
        advance_width_max: UfWord::new(ADVANCE_WIDTH),
        ..Default::default()
    };
    builder.add_table(&hhea)?;

    // maxp
    let maxp = Maxp::new(num_glyphs);
    builder.add_table(&maxp)?;

    // OS/2
    let os2 = Os2 {
        s_typo_ascender: ASCENT,
        s_typo_descender: -DESCENT,
        s_typo_line_gap: 0,
        us_win_ascent: ASCENT as u16,
        us_win_descent: DESCENT as u16,
        us_weight_class: 400,
        us_width_class: 5,
        sx_height: Some(0),
        s_cap_height: Some(0),
        ul_code_page_range_1: Some(0),
        ul_code_page_range_2: Some(0),
        us_default_char: Some(0),
        us_break_char: Some(0x0020),
        us_max_context: Some(0),
        ..Default::default()
    };
    builder.add_table(&os2)?;

    // post (format 3 - no glyph names)
    let post = Post::new(
        Fixed::from_f64(0.0),   // italic_angle
        FWord::new(-100),       // underline_position
        FWord::new(50),         // underline_thickness
        0,                      // is_fixed_pitch
        0, 0, 0, 0,            // mem fields
    );
    builder.add_table(&post)?;

    // name table
    // Platform 3 (Windows), encoding 1 (Unicode BMP), language 0x0409 (English US)
    let name = Name::new(vec![
        NameRecord::new(3, 1, 0x0409, NameId::FAMILY_NAME, FAMILY_NAME.to_string().into()),
        NameRecord::new(3, 1, 0x0409, NameId::SUBFAMILY_NAME, "Regular".to_string().into()),
        NameRecord::new(3, 1, 0x0409, NameId::FULL_NAME, FAMILY_NAME.to_string().into()),
        NameRecord::new(3, 1, 0x0409, NameId::POSTSCRIPT_NAME, "AppleColorEmoji".to_string().into()),
    ]);
    builder.add_table(&name)?;

    // cmap - format 12 (full 32-bit Unicode coverage)
    let groups: Vec<SequentialMapGroup> = font_data.single_cp
        .iter()
        .enumerate()
        .map(|(i, (cp, _))| SequentialMapGroup::new(*cp, *cp, (i + 1) as u32))
        .collect();

    let subtable = CmapSubtable::format_12(0, groups);
    // Platform 0 (Unicode), encoding 4 (Unicode full)
    let cmap = Cmap::new(vec![
        EncodingRecord::new(PlatformId::Unicode, 4, subtable),
    ]);
    builder.add_table(&cmap)?;

    // hmtx
    let h_metrics: Vec<LongMetric> = (0..num_glyphs)
        .map(|_| LongMetric::new(ADVANCE_WIDTH, 0))
        .collect();
    let hmtx = Hmtx::new(h_metrics, vec![]);
    builder.add_table(&hmtx)?;

    // glyf + loca (empty - sbix fonts have no outlines)
    let empty_glyf: Vec<u8> = vec![];
    let loca: Vec<u8> = vec![0u8; (num_glyphs as usize + 1) * 4];
    builder.add_raw(Tag::new(b"glyf"), empty_glyf);
    builder.add_raw(Tag::new(b"loca"), loca);

    // sbix
    builder.add_raw(Tag::new(b"sbix"), sbix_bytes);

    let font_bytes = builder.build();
    std::fs::write(output, &font_bytes)
        .with_context(|| format!("Failed to write {output}"))?;

    let size_mb = font_bytes.len() as f64 / (1024.0 * 1024.0);
    println!("\nSaved: {output} ({size_mb:.1} MB)");
    println!("  Family: {FAMILY_NAME}");
    println!("  Strikes: {sizes:?}");
    println!("  cmap entries: {}", font_data.single_cp.len());

    Ok(())
}
