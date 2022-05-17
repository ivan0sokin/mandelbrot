#[derive(Default)]
pub struct Gray<T> {
    value: T
}

#[derive(Default)]
pub struct RGB<T> {
    r: T,
    g: T,
    b: T
}

impl<T> Gray<T> {
    pub fn new(value: T) -> Self {
        Self {
            value
        }
    }

}

impl Gray<f64> {
    pub fn to_u8(&self) -> u8 {
        (self.value * 255.0) as u8
    }
}

impl<T: Copy> RGB<T> {
    pub fn new(r: T, g: T, b: T) -> Self {
        Self {
            r,
            g,
            b
        }
    }

    pub fn from_scalar(scalar: T) -> Self {
        Self {
            r: scalar,
            g: scalar,
            b: scalar
        }
    }
}

impl RGB<f64> {
    pub fn to_bytes(&self) -> [u8; 3] {
        [(self.r * 255.0) as u8, (self.g * 255.0) as u8, (self.b * 255.0) as u8]
    }
}

impl RGB<u8> {
    pub fn to_zero_to_one_interval_rgb(&self) -> RGB<f64> {
        RGB {
            r: self.r as f64 / 255.0,
            g: self.g as f64 / 255.0,
            b: self.b as f64 / 255.0
        }
    }
}