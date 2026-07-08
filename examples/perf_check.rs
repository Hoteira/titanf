//! Quick timing sanity check: titanf vs fontdue on a system font.

use std::hint::black_box;
use std::time::Instant;
use titanf::TrueTypeFont;

fn main() {
    let data = std::fs::read("NotoSansSC-Medium.ttf").unwrap();
    let mut font = TrueTypeFont::load_font(&data).unwrap();
    let fd = fontdue::Font::from_bytes(&data as &[u8], Default::default()).unwrap();

    let chars: Vec<char> =
        "你好世界人大天地中国日本语文字学生说话书看听写读吃喝走来去做ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789.,;:!?@#$%&*()"
            .chars()
            .collect();
    let count = 10_000;

    for &size in &[12.0f32, 24.0, 72.0, 250.0] {
        // warmup
        for i in 0..1000 {
            black_box(font.get_char::<false>(chars[i % chars.len()], size));
        }
        let t = Instant::now();
        for i in 0..count {
            black_box(font.get_char::<false>(black_box(chars[i % chars.len()]), size));
        }
        let titanf_ms = t.elapsed().as_secs_f64() * 1000.0;

        for i in 0..1000 {
            black_box(fd.rasterize(chars[i % chars.len()], size));
        }
        let t = Instant::now();
        for i in 0..count {
            black_box(fd.rasterize(black_box(chars[i % chars.len()]), size));
        }
        let fontdue_ms = t.elapsed().as_secs_f64() * 1000.0;

        println!(
            "{:>5}px  titanf: {:>8.2} ms   fontdue: {:>8.2} ms   ratio: {:.2}x",
            size,
            titanf_ms,
            fontdue_ms,
            titanf_ms / fontdue_ms
        );
    }
}
