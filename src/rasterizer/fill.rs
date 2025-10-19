use crate::F32NoStd;

pub(crate) fn fill_span(bitmap: &mut [u8], y: usize, width: usize, start_x: f32, end_x: f32) {
    let start = start_x.floor() as usize;
    let end = end_x.floor() as usize;

    for i in start..end {
        bitmap[y * width + i] = 255;
    }
}
