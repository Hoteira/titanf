use titanf::TrueTypeFont;

pub fn main() {
    let font_data = include_bytes!("../Roboto-Medium.ttf");
    let mut font = TrueTypeFont::load_font(font_data);

    font.set_dpi(96.0); //Set your screen's DPI (Default is 72)

    let (metrics, bitmap) = font.get_char::<false>('A', 32.0);
    //                                                              ^^^^^ cache disabled

    let kerning = font.get_kerning('A', 'B');
    //Only works with (mostly outdated) fonts that have a kern table
}