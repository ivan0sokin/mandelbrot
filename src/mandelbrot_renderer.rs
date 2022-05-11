use crate::complex::Complex;

type FloatType = f64;

#[derive(Clone)]
pub struct MandelbrotRenderer {
    canvas_bounds: (usize, usize),
    plane_bounds: (Complex<FloatType>, Complex<FloatType>),
    iteration_count: usize
}

impl MandelbrotRenderer {
    pub const DEFAULT_PLANE_BOUNDS: (Complex<FloatType>, Complex<FloatType>) = (Complex{ re: -3.0, im: 1.0}, Complex{ re: 1.0, im: -1.0});
    const MAX_SQUARED_MAGNITUDE: FloatType = 4.0;

    pub fn new(canvas_bounds: (usize, usize), plane_bounds: (Complex<FloatType>, Complex<FloatType>), iteration_count: usize) -> Self {
        Self {
            canvas_bounds,
            plane_bounds,
            iteration_count
        }
    }

    pub fn render(&self, buf: &mut [u8], offset: usize) {
        for i in 0..buf.len() {
            let j = offset + i;
            let pixel = (j % self.canvas_bounds.0, self.canvas_bounds.1 - j / self.canvas_bounds.0);
            if let Some(c) = self.escape_time(&self.map_pixel_to_complex_plane(&pixel)) {
                buf[i] = 255u8 - (c as f64 / self.iteration_count as f64 * (255.0 - f64::EPSILON)) as u8;
            } else {
                buf[i] = 0;
            }
        }
    }

    fn escape_time(&self, point: &Complex<FloatType>) -> Option<usize> {
        let mut z = Complex::default();
        for i in 0..self.iteration_count {
            if z.squared_magnitude() > MandelbrotRenderer::MAX_SQUARED_MAGNITUDE {
                return Some(i);
            }

            z = z * z + *point;
        }

        None
    }

    fn map_pixel_to_complex_plane(&self, pixel: &(usize, usize)) -> Complex<FloatType> {
        let pixel = (pixel.0 as FloatType, pixel.1 as FloatType);
        let canvas_bounds = (self.canvas_bounds.0 as FloatType, self.canvas_bounds.1 as FloatType);
        let re = self.plane_bounds.0.re + (self.plane_bounds.1.re - self.plane_bounds.0.re) * pixel.0 / canvas_bounds.0;
        let im = self.plane_bounds.1.im + (self.plane_bounds.0.im - self.plane_bounds.1.im) * pixel.1 / canvas_bounds.1;
        Complex::new(re, im)
    }
    
    pub fn get_canvas_resolution(&self) -> usize {
        self.canvas_bounds.0 * self.canvas_bounds.1
    }
}