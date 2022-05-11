use std::ops::{Add, Sub, Mul};

#[derive(Copy, Clone, Default)]
pub struct Complex<T> {
    pub re: T,
    pub im: T
}

impl<T> Complex<T> {
    pub fn new(re: T, im: T) -> Self {
        Self {
            re,
            im
        }
    }

    pub fn squared_magnitude(&self) -> T where T: Copy + Mul<Output = T> + Add<Output = T> {
        self.re * self.re + self.im * self.im
    }
}

impl<T: Add<Output = T>> Add for Complex<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            re: self.re + rhs.re,
            im: self.im + rhs.im
        }
    }
}

impl<T: Copy + Add<Output = T> + Sub<Output = T> + Mul<Output = T>> Mul for Complex<T> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            re: self.re * rhs.re - self.im * rhs.im,
            im: self.re * rhs.im + self.im * rhs.re
        }
    }
}