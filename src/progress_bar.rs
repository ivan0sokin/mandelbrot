use std::io::Write;
use crate::output::Output;

pub struct ProgressBar<W: Write + Send + Sync> {
    symbol: char,
    len: usize,
    output: Output<W>
}

impl<W: Write + Send + Sync> ProgressBar<W> {
    pub fn new(symbol: char, len: usize, output: Output<W>) -> Self {
        Self {
            symbol,
            len,
            output
        }
    }

    pub fn print_progress(&mut self, done: f64) {
        let repeat_count = (done * self.len as f64) as usize;
        self.output.write_text_flushed(&format!("\r[{0: <1$}] {2:.1}% rendered", String::from(self.symbol).repeat(repeat_count), self.len, done * 100.0));
    }
}