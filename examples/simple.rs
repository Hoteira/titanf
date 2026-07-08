use titanf::TrueTypeFont;

pub fn main() {
    // Any TrueType font works; drop one next to Cargo.toml.
    // (Read at runtime so the crate packages without bundling a font.)
    let font_data = std::fs::read("NotoSansSC-Medium.ttf")
        .expect("place a font at ./NotoSansSC-Medium.ttf to run this example");
    let mut font = TrueTypeFont::load_font(&font_data).unwrap();

    let (metrics, bitmap) = font.get_char::<false>('@', 32.0);
    println!(
        "kerning('T','o') = {:?} font units",
        font.get_kerning('T', 'o')
    );

    let chars = b" .:-=+*#%@";
    for i in 0..metrics.height {
        for j in 0..metrics.width {
            let alpha = bitmap[i * metrics.width + j];
            let idx = (alpha as usize * (chars.len() - 1)) / 255;
            print!("{}", chars[idx] as char);
        }
        println!();
    }
}
