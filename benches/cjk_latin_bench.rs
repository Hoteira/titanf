use ab_glyph::{Font, PxScale, ScaleFont};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rusttype::{point, Scale};
use std::hint::black_box;
use titanf::TrueTypeFont;

fn benchmark_cjk_latin(c: &mut Criterion) {
    let mut group = c.benchmark_group("cjk_latin_scaling");

    // Configure group
    group.sample_size(10);
    group.warm_up_time(std::time::Duration::from_secs(3));
    group.measurement_time(std::time::Duration::from_secs(3));

    // Load fonts
    let font_data = std::fs::read("NotoSansSC-Medium.ttf").expect("Failed to load font");
    let mut font_0 = TrueTypeFont::load_font(&font_data);
    let font_1 = fontdue::Font::from_bytes(&font_data as &[u8], Default::default()).unwrap();
    let font_2 = rusttype::Font::try_from_vec(font_data.clone()).unwrap();
    let font_3 = ab_glyph::FontRef::try_from_slice(&font_data).unwrap();

    // CJK characters (sample from common ranges)
    let cjk_chars = vec![
        // Common CJK Unified Ideographs
        '你', '好', '世', '界', '人', '大', '天', '地', '中', '国',
        '日', '本', '语', '文', '字', '学', '生', '说', '话', '书',
        '看', '听', '写', '读', '吃', '喝', '走', '来', '去', '做',
        // Hiragana
        'あ', 'い', 'う', 'え', 'お', 'か', 'き', 'く', 'け', 'こ',
        'さ', 'し', 'す', 'せ', 'そ', 'た', 'ち', 'つ', 'て', 'と',
        // Katakana
        'ア', 'イ', 'ウ', 'エ', 'オ', 'カ', 'キ', 'ク', 'ケ', 'コ',
        'サ', 'シ', 'ス', 'セ', 'ソ', 'タ', 'チ', 'ツ', 'テ', 'ト',
        // Hangul
        '가', '나', '다', '라', '마', '바', '사', '아', '자', '차',
        '한', '국', '어', '학', '교', '선', '생', '님', '안', '녕',
    ];

    // Latin characters (A-Z, a-z, common symbols)
    let latin_chars = vec![
        'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M',
        'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
        'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
        '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '-', '=', '+',
        '[', ']', '{', '}', '|', '\\', ';', ':', '\'', '"', ',', '.',
        '<', '>', '/', '?', '~', '`', ' ',
    ];

    // Combine CJK and Latin
    let mut all_chars = Vec::new();
    all_chars.extend_from_slice(&cjk_chars);
    all_chars.extend_from_slice(&latin_chars);

    let sizes = [12, 16, 24, 48, 72, 120, 250];
    let count = 10_000;

    for &size in &sizes {
        let bench_name = format!("{}pt_1000chars", size);

        // TitanF benchmark
        group.bench_with_input(
            BenchmarkId::new("titanf", &bench_name),
            &(size, count),
            |b, &(size, count)| {
                b.iter(|| {
                    for i in 0..count {
                        let c = all_chars[i % all_chars.len()];
                        let (metrics, bitmap) = font_0.get_char::<false>(
                            black_box(c),
                            black_box(size as f32),
                        );
                        black_box(&bitmap);
                    }
                }
                );
            },
        );

        //rusttype
        group.bench_with_input(
            BenchmarkId::new("rusttype", &bench_name),
            &(size, count),
            |b, &(size, count)| {
                b.iter(|| {
                    for i in 0..count {
                        let c = all_chars[i % all_chars.len()];
                        let glyph = font_2.glyph(black_box(c))
                            .scaled(Scale::uniform(black_box(size as f32)))
                            .positioned(point(0.0, 0.0));

                        if let Some(bb) = glyph.pixel_bounding_box() {
                            let mut bitmap = vec![0u8; (bb.width() * bb.height()) as usize];
                            glyph.draw(|x, y, v| {
                                let idx = (y * bb.width() as u32 + x) as usize;
                                if idx < bitmap.len() {
                                    bitmap[idx] = (v * 255.0) as u8;
                                }
                            });
                            black_box(&bitmap);
                        }
                    }
                },
                );
            },
        );

        // fontdue benchmark
        group.bench_with_input(
            BenchmarkId::new("fontdue", &bench_name),
            &(size, count),
            |b, &(size, count)| {
                b.iter(|| {
                    for i in 0..count {
                        let c = all_chars[i % all_chars.len()];
                        let (metrics, bitmap) = font_1.rasterize(
                            black_box(c),
                            black_box(size as f32),
                        );
                        black_box(&bitmap);
                    }
                }
                );
            },
        );

        // ab_glyph benchmark
        group.bench_with_input(
            BenchmarkId::new("ab_glyph", &bench_name),
            &(size, count),
            |b, &(size, count)| {
                b.iter(|| {
                    for i in 0..count {
                        let c = all_chars[i % all_chars.len()];
                        let scaled_font = font_3.as_scaled(PxScale::from(black_box(size as f32)));
                        let glyph = scaled_font.scaled_glyph(black_box(c));

                        if let Some(outlined) = font_3.outline_glyph(glyph) {
                            let bounds = outlined.px_bounds();
                            let width = bounds.width() as usize;
                            let height = bounds.height() as usize;
                            let mut bitmap = vec![0u8; width * height];

                            outlined.draw(|x, y, v| {
                                let idx = y as usize * width + x as usize;
                                if idx < bitmap.len() {
                                    bitmap[idx] = (v * 255.0) as u8;
                                }
                            });
                            black_box(&bitmap);
                        }
                    }
                }
                );
            },
        );
    }

    group.finish();
}

criterion_group!(benches, benchmark_cjk_latin);
criterion_main!(benches);