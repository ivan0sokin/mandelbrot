pub struct GrayscaleEncoder;

impl GrayscaleEncoder {
    pub fn new<W: std::io::Write>(w: &mut W, bounds: (usize, usize)) -> png::Encoder<&mut W> {
        let mut encoder = png::Encoder::new(w, bounds.0 as u32, bounds.1 as u32);
        encoder.set_color(png::ColorType::Grayscale);
        encoder.set_depth(png::BitDepth::Eight);

        encoder
    }
}