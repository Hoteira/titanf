use titanf::TrueTypeFont;

fn push_u16(v: &mut Vec<u8>, x: u16) {
    v.extend_from_slice(&x.to_be_bytes());
}

fn push_u32(v: &mut Vec<u8>, x: u32) {
    v.extend_from_slice(&x.to_be_bytes());
}

fn push_i16(v: &mut Vec<u8>, x: i16) {
    v.extend_from_slice(&x.to_be_bytes());
}

struct TableEntry {
    tag: [u8; 4],
    data: Vec<u8>,
}

/// Build a minimal but structurally valid TrueType font containing:
/// glyph 0 = empty (missing glyph), glyph 1 = a 500x500 unit square
/// mapped to 'A' through a cmap format 4 subtable. units_per_em = 1000.
fn build_test_font() -> Vec<u8> {
    // --- head ---
    let mut head = Vec::new();
    push_u16(&mut head, 1); // major
    push_u16(&mut head, 0); // minor
    push_u32(&mut head, 0x0001_0000); // revision
    push_u32(&mut head, 0); // checksum adjustment
    push_u32(&mut head, 0x5F0F_3CF5); // magic
    push_u16(&mut head, 0); // flags
    push_u16(&mut head, 1000); // units_per_em
    head.extend_from_slice(&[0u8; 16]); // created + modified
    push_i16(&mut head, 0); // x_min
    push_i16(&mut head, 0); // y_min
    push_i16(&mut head, 500); // x_max
    push_i16(&mut head, 500); // y_max
    push_u16(&mut head, 0); // mac style
    push_u16(&mut head, 8); // lowest rec ppem
    push_i16(&mut head, 2); // font direction hint
    push_i16(&mut head, 0); // index_to_loc_format: short
    push_i16(&mut head, 0); // glyph data format

    // --- maxp ---
    let mut maxp = Vec::new();
    push_u32(&mut maxp, 0x0001_0000); // version
    push_u16(&mut maxp, 2); // num_glyphs
    maxp.extend_from_slice(&[0u8; 26]); // remaining maxima

    // --- hhea ---
    let mut hhea = Vec::new();
    push_u16(&mut hhea, 1); // major
    push_u16(&mut hhea, 0); // minor
    push_i16(&mut hhea, 800); // ascender
    push_i16(&mut hhea, -200); // descender
    push_i16(&mut hhea, 0); // line gap
    push_u16(&mut hhea, 600); // advance width max
    hhea.extend_from_slice(&[0u8; 22]); // bearings .. metric data format
    push_u16(&mut hhea, 2); // number_of_h_metrics

    // --- hmtx --- (2 long metrics)
    let mut hmtx = Vec::new();
    push_u16(&mut hmtx, 600); // glyph 0 advance
    push_i16(&mut hmtx, 0); // glyph 0 lsb
    push_u16(&mut hmtx, 600); // glyph 1 advance
    push_i16(&mut hmtx, 0); // glyph 1 lsb

    // --- glyf: glyph 1 = closed square contour (0,0)-(500,500) ---
    let mut glyf = Vec::new();
    push_i16(&mut glyf, 1); // one contour
    push_i16(&mut glyf, 0); // x_min
    push_i16(&mut glyf, 0); // y_min
    push_i16(&mut glyf, 500); // x_max
    push_i16(&mut glyf, 500); // y_max
    push_u16(&mut glyf, 3); // end point of contour 0 (4 points)
    push_u16(&mut glyf, 0); // instruction length
    // flags: 4 points, all on-curve (0x01), x/y as signed words (no short bits)
    glyf.extend_from_slice(&[0x01, 0x01, 0x01, 0x01]);
    // x deltas: 0, 500, 0, -500
    for d in [0i16, 500, 0, -500] {
        push_i16(&mut glyf, d);
    }
    // y deltas: 0, 0, 500, 0  -> points (0,0) (500,0) (500,500) (0,500)
    for d in [0i16, 0, 500, 0] {
        push_i16(&mut glyf, d);
    }

    // --- loca (short format: offsets / 2), glyph 0 empty ---
    let mut loca = Vec::new();
    push_u16(&mut loca, 0); // glyph 0 start
    push_u16(&mut loca, 0); // glyph 1 start (glyph 0 is empty)
    push_u16(&mut loca, (glyf.len() / 2) as u16); // end

    // --- cmap: format 4, 'A' (0x41) -> glyph 1 ---
    let mut sub4 = Vec::new();
    push_u16(&mut sub4, 4); // format
    // 2 segments: [0x41..0x41] and the required terminator [0xFFFF]
    let seg_count_x2 = 4u16;
    push_u16(&mut sub4, 0); // length (patched below)
    push_u16(&mut sub4, 0); // language
    push_u16(&mut sub4, seg_count_x2);
    push_u16(&mut sub4, 4); // search range
    push_u16(&mut sub4, 1); // entry selector
    push_u16(&mut sub4, 0); // range shift
    push_u16(&mut sub4, 0x41); // end_count[0]
    push_u16(&mut sub4, 0xFFFF); // end_count[1]
    push_u16(&mut sub4, 0); // reserved pad
    push_u16(&mut sub4, 0x41); // start_count[0]
    push_u16(&mut sub4, 0xFFFF); // start_count[1]
    push_i16(&mut sub4, (1i32 - 0x41) as i16); // id_delta[0]: 'A' -> glyph 1
    push_i16(&mut sub4, 1); // id_delta[1]
    push_u16(&mut sub4, 0); // id_range_offset[0]
    push_u16(&mut sub4, 0); // id_range_offset[1]
    let sub4_len = sub4.len() as u16;
    sub4[2..4].copy_from_slice(&sub4_len.to_be_bytes());

    let mut cmap = Vec::new();
    push_u16(&mut cmap, 0); // version
    push_u16(&mut cmap, 1); // one encoding record
    push_u16(&mut cmap, 3); // platform: Windows
    push_u16(&mut cmap, 1); // encoding: Unicode BMP
    push_u32(&mut cmap, 12); // subtable offset (4 header + 8 record)
    cmap.extend_from_slice(&sub4);

    // --- assemble ---
    let tables = vec![
        TableEntry {
            tag: *b"cmap",
            data: cmap,
        },
        TableEntry {
            tag: *b"glyf",
            data: glyf,
        },
        TableEntry {
            tag: *b"head",
            data: head,
        },
        TableEntry {
            tag: *b"hhea",
            data: hhea,
        },
        TableEntry {
            tag: *b"hmtx",
            data: hmtx,
        },
        TableEntry {
            tag: *b"loca",
            data: loca,
        },
        TableEntry {
            tag: *b"maxp",
            data: maxp,
        },
    ];

    let num_tables = tables.len() as u16;
    let mut font = Vec::new();
    push_u32(&mut font, 0x0001_0000); // scaler type
    push_u16(&mut font, num_tables);
    push_u16(&mut font, 0); // search range
    push_u16(&mut font, 0); // entry selector
    push_u16(&mut font, 0); // range shift

    let mut data_offset = 12 + tables.len() * 16;
    let mut body = Vec::new();
    for t in &tables {
        font.extend_from_slice(&t.tag);
        push_u32(&mut font, 0); // checksum (unchecked by the parser)
        push_u32(&mut font, data_offset as u32);
        push_u32(&mut font, t.data.len() as u32);
        body.extend_from_slice(&t.data);
        // 4-byte align the next table
        let pad = (4 - t.data.len() % 4) % 4;
        body.extend_from_slice(&vec![0u8; pad]);
        data_offset += t.data.len() + pad;
    }
    font.extend_from_slice(&body);
    font
}

// ---------------------------------------------------------------------------
// Parsing & rendering
// ---------------------------------------------------------------------------

#[test]
fn parses_synthetic_font() {
    let bytes = build_test_font();
    assert!(TrueTypeFont::load_font(&bytes).is_ok());
}

#[test]
fn renders_square_glyph() {
    let bytes = build_test_font();
    let mut font = TrueTypeFont::load_font(&bytes).unwrap();

    // 500/1000 em square at 100px -> ~50x50 bitmap
    let (metrics, bitmap) = font.get_char::<false>('A', 100.0);
    assert_eq!(bitmap.len(), metrics.width * metrics.height);
    assert!(
        (49..=51).contains(&metrics.width),
        "width {}",
        metrics.width
    );
    assert!(
        (49..=51).contains(&metrics.height),
        "height {}",
        metrics.height
    );
    assert_eq!(metrics.advance_width, 60); // 600/1000 em * 100px

    // The square's interior must be fully opaque and its center row must
    // have no coverage holes.
    let cx = metrics.width / 2;
    let cy = metrics.height / 2;
    assert_eq!(bitmap[cy * metrics.width + cx], 255);
    for x in 1..metrics.width - 1 {
        assert_eq!(bitmap[cy * metrics.width + x], 255, "hole at x={}", x);
    }
}

#[test]
fn unmapped_char_falls_back_to_missing_glyph() {
    let bytes = build_test_font();
    let mut font = TrueTypeFont::load_font(&bytes).unwrap();

    // 'Z' is not mapped; glyph 0 is empty, so this renders nothing but
    // must not panic and must report consistent metrics.
    let (metrics, bitmap) = font.get_char::<false>('Z', 24.0);
    assert_eq!(bitmap.len(), metrics.width * metrics.height);
    assert_eq!(metrics.width, 0);
    assert_eq!(metrics.height, 0);
}

#[test]
fn rendering_is_deterministic() {
    let bytes = build_test_font();
    let mut font = TrueTypeFont::load_font(&bytes).unwrap();
    let a = font.get_char::<false>('A', 33.5);
    let b = font.get_char::<false>('A', 33.5);
    assert_eq!(a.1, b.1);
}

// ---------------------------------------------------------------------------
// Robustness: no malformed input may panic
// ---------------------------------------------------------------------------

#[test]
fn truncated_fonts_never_panic() {
    let bytes = build_test_font();
    for len in 0..bytes.len() {
        // Err is fine, Ok is fine - panicking is not. If load succeeds on a
        // truncated prefix, rendering from it must not panic either.
        if let Ok(mut font) = TrueTypeFont::load_font(&bytes[..len]) {
            let _ = font.get_char::<false>('A', 16.0);
            let _ = font.get_char::<false>('Z', 16.0);
        }
    }
}

#[test]
fn corrupted_fonts_never_panic() {
    let base = build_test_font();

    // Flip bytes to stress offsets, counts and flags. Every byte position,
    // a few adversarial values each.
    for pos in 0..base.len() {
        for val in [0x00u8, 0xFF, 0x7F, 0x80] {
            let mut bytes = base.clone();
            if bytes[pos] == val {
                continue;
            }
            bytes[pos] = val;
            if let Ok(mut font) = TrueTypeFont::load_font(&bytes) {
                let _ = font.get_char::<false>('A', 16.0);
                let _ = font.get_char::<false>('Z', 16.0);
            }
        }
    }
}

#[test]
fn empty_and_garbage_input() {
    assert!(TrueTypeFont::load_font(&[]).is_err());
    assert!(TrueTypeFont::load_font(&[0u8; 4]).is_err());
    let garbage: Vec<u8> = (0..1024u32).map(|i| (i * 7 + 13) as u8).collect();
    // Random bytes may parse as an (unusable) font or fail; either way, no
    // panic, and a successful parse must render without panicking.
    if let Ok(mut font) = TrueTypeFont::load_font(&garbage) {
        let _ = font.get_char::<false>('A', 12.0);
    }
}

// ---------------------------------------------------------------------------
// Cache behaviour
// ---------------------------------------------------------------------------

#[test]
fn cache_distinguishes_fractional_sizes() {
    let bytes = build_test_font();
    let mut font = TrueTypeFont::load_font(&bytes).unwrap();

    // Two sizes that share ceil(size) - the old cache keyed on ceil and
    // served one bitmap for both. Coverage (AA edges) must differ.
    let (_, b1) = font.get_char::<true>('A', 16.2);
    let (_, b2) = font.get_char::<true>('A', 17.0);
    assert_ne!(b1, b2, "16.2px and 17.0px must not share a cache entry");

    // A repeated request must serve the identical bitmap.
    let (_, b1_again) = font.get_char::<true>('A', 16.2);
    assert_eq!(b1, b1_again);
}

#[test]
fn cache_hit_matches_uncached_render() {
    let bytes = build_test_font();
    let mut font = TrueTypeFont::load_font(&bytes).unwrap();

    let (_, cold) = font.get_char::<true>('A', 21.7); // populates cache
    let (_, warm) = font.get_char::<true>('A', 21.7); // served from cache
    let (_, uncached) = font.get_char::<false>('A', 21.7);
    assert_eq!(cold, warm);
    assert_eq!(cold, uncached);
}

#[test]
fn set_dpi_invalidates_cache() {
    let bytes = build_test_font();
    let mut font = TrueTypeFont::load_font(&bytes).unwrap();

    let (m1, _) = font.get_char::<true>('A', 20.0);
    font.set_dpi(144.0); // doubles effective scale
    let (m2, _) = font.get_char::<true>('A', 20.0);
    assert!(
        m2.width > m1.width,
        "cache served a stale bitmap after set_dpi: {} !> {}",
        m2.width,
        m1.width
    );
}

// ---------------------------------------------------------------------------
// Kerning API surface
// ---------------------------------------------------------------------------

#[test]
fn kerning_absent_is_none() {
    let bytes = build_test_font(); // no kern table
    let font = TrueTypeFont::load_font(&bytes).unwrap();
    assert!(font.get_kerning('A', 'A').is_none());
}
