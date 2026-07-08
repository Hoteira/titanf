//! Kerning sanity check against real fonts (legacy `kern` + GPOS).

use titanf::TrueTypeFont;

fn check(path: &str, pairs: &[(char, char)]) {
    let Ok(data) = std::fs::read(path) else {
        println!("{}: not found, skipped", path);
        return;
    };
    let mut font = TrueTypeFont::load_font(&data).unwrap();
    println!("{}:", path);
    for &(l, r) in pairs {
        println!("  kern('{}','{}') = {:?}", l, r, font.get_kerning(l, r));
    }
    // Indexed rendering: resolve a char to its glyph id, render by id.
    let a = font.lookup_glyph_index('A');
    let (m, bm) = font.get_indexed::<false>(a, 24.0);
    println!("  glyph_index('A') = {}, indexed render {}x{}, {} bytes", a, m.width, m.height, bm.len());
}

fn main() {
    check(
        "C:\\Windows\\Fonts\\arial.ttf",
        &[('A', 'V'), ('T', 'o'), ('Y', 'o'), ('L', 'T'), ('A', 'A')],
    );
    check(
        "C:\\Windows\\Fonts\\times.ttf",
        &[('A', 'V'), ('T', 'o'), ('Y', 'o'), ('A', 'A')],
    );
    check("NotoSansSC-Medium.ttf", &[('A', 'V'), ('T', 'o'), ('Y', 'o'), ('A', 'A')]);
}
