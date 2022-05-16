pub struct GrayscaleEncoder;

impl<'a> GrayscaleEncoder {
    pub fn new<W: std::io::Write>(w: W, bounds: (usize, usize)) -> png::Encoder<'a, W> {
        let mut encoder = png::Encoder::new(w, bounds.0 as u32, bounds.1 as u32);
        encoder.set_color(png::ColorType::Grayscale);
        encoder.set_depth(png::BitDepth::Eight);

        encoder
    }
}