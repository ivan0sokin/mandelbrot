use crate::complex::Complex;
use crate::color::{Gray, RGB};

pub type FloatType = f64;

#[derive(Clone)]
pub struct MandelbrotRenderer {
    canvas_bounds: (usize, usize),
    plane_bounds: (Complex<FloatType>, Complex<FloatType>),
    iteration_count: usize
}

impl MandelbrotRenderer {
    pub const DEFAULT_PLANE_BOUNDS: (Complex<FloatType>, Complex<FloatType>) = (Complex{re: -3.0, im: 1.0}, Complex{re: 1.0, im: -1.0});
    pub const WIKI_PLANE_BOUNDS: (Complex<FloatType>, Complex<FloatType>) = (Complex{re: -2.23845, im: 1.2}, Complex{re: 0.83845, im: -1.2});
    const MAX_SQUARED_MAGNITUDE: FloatType = 4.0;

    pub fn new(canvas_bounds: (usize, usize), plane_bounds: (Complex<FloatType>, Complex<FloatType>), iteration_count: usize) -> Self {
        Self {
            canvas_bounds,
            plane_bounds,
            iteration_count
        }
    }

    pub fn render(&self, buf: &mut [u8], offset: usize, get_color: impl Fn(&Complex<FloatType>, Option<usize>) -> RGB<FloatType>) {
        for i in 0..buf.len() / 3 {
            let j = offset / 3 + i;
            let pixel = (j % self.canvas_bounds.0, self.canvas_bounds.1 - j / self.canvas_bounds.0 - 1);
            let z = self.map_pixel_to_complex_plane(&pixel);
            let color = get_color(&z, self.escape_time(&z));
            buf[3 * i..3 * i + 3].copy_from_slice(&color.to_bytes());
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