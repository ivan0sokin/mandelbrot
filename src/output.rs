use std::io::Write;

pub struct Output<W: Write> {
    inner: W
}

impl<W: Write> Output<W> {
    pub fn new(output: W) -> Self {
        Self {
            inner: output
        }
    }

    pub fn write_line_flushed(&mut self, line: &str) {
        self.write_line(line);
        self.flush();
    }

    pub fn write_line(&mut self, line: &str) {
        self.write_text(line);
        self.write_text("\n");
    }

    pub fn write_text_flushed(&mut self, text: &str) {
        self.write_text(text);
        self.flush();
    }

    pub fn write_text(&mut self, text: &str) {
        self.inner.write_all(text.as_bytes()).unwrap();
    }

    pub fn flush(&mut self) {
        self.inner.flush().unwrap();
    }
}