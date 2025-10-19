use titanf::TrueTypeFont;
use std::time::Instant;

pub fn main() {
    let font_data = include_bytes!("../Roboto-Medium.ttf");
    let mut font = TrueTypeFont::load_font(font_data);

    let t0 = Instant::now();
    let (metrics, bitmap) = font.get_char::<false>('a', 32);
    //                                                              ^^^^^ cache disabled
    let e = t0.elapsed();
    println!("Time: {:?}", e);

    println!("-----------------------------------------------");
    for i in 0..metrics.height {
        for j in 0..metrics.width {
            print!("{:02x} ", bitmap[i * metrics.width + j]);
        }
        println!();
    }

    let kerning = font.get_kerning('A', 'B');
    //Only works with fonts that have a kern table
}