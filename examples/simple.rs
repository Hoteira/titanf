use titanf::TrueTypeFont;

pub fn main() {
    let font_data = include_bytes!("../CaskaydiaMonoNerdFontMono-Regular.ttf");
    let mut font = TrueTypeFont::load_font(font_data).unwrap();

    let (metrics, bitmap) = font.get_char::<false>('@', 12.0);

    let kern = font.get_kerning('a', 'b');

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
