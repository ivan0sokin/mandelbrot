use std::fmt::{Display, Result, Formatter};

pub struct ProgressBar {
    symbol: char,
    len: usize,
    done: f64
}

impl ProgressBar {
    pub fn new(symbol: char, len: usize) -> Self {
        Self {
            symbol,
            len,
            done: 0.0
        }
    }

    pub fn update(&mut self, done: f64) {
        self.done = done;
    }
}

impl Display for ProgressBar {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let repeat_count = (self.done * self.len as f64) as usize;
        write!(f, "\r[{0: <1$}] {2:.1}% rendered", String::from(self.symbol).repeat(repeat_count), self.len, self.done * 100.0)
    }
}