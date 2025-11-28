use titanf::TrueTypeFont;

pub fn main() {
    let font_data = include_bytes!("../CaskaydiaMonoNerdFontMono-Regular.ttf");
    let mut font = TrueTypeFont::load_font(font_data).unwrap();

    let (metrics, bitmap) = font.get_char::<false>('a', 16.0);

    let kern = font.get_kerning('a', 'b');
}
