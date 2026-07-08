//! Visual smoke test: renders bug-prone glyphs at sizes covering both the
//! baked tier (<= 40px) and the per-size flatten tier, as ASCII art.

use titanf::TrueTypeFont;

fn show(font: &mut TrueTypeFont, c: char, size: f32) {
    let (metrics, bitmap) = font.get_char::<false>(c, size);
    let mut haze = 0usize;
    let mut nonzero = 0usize;
    for &v in &bitmap {
        if v > 0 {
            nonzero += 1;
            if v < 8 {
                haze += 1;
            }
        }
    }
    println!(
        "'{}' @{}px  {}x{}  nonzero {}  faint(1-7) {}",
        c, size, metrics.width, metrics.height, nonzero, haze
    );
    let chars = b" .:-=+*#%@";
    for i in 0..metrics.height {
        let mut row = String::new();
        for j in 0..metrics.width {
            let alpha = bitmap[i * metrics.width + j];
            let idx = (alpha as usize * (chars.len() - 1)) / 255;
            row.push(chars[idx] as char);
        }
        println!("{}", row);
    }
    println!();
}

fn main() {
    let data = std::fs::read("C:\\Windows\\Fonts\\arial.ttf").unwrap();
    let mut font = TrueTypeFont::load_font(&data).unwrap();

    show(&mut font, 'q', 24.0); // fill-leak regression case (baked tier)
    show(&mut font, 'q', 64.0); // fill-leak regression case (flatten tier)
    show(&mut font, 'o', 40.0); // tier boundary, below
    show(&mut font, 'o', 41.0); // tier boundary, above
    show(&mut font, 'n', 64.0); // background-veil regression case
    show(&mut font, '@', 16.0); // small curved glyph, baked tier
}
