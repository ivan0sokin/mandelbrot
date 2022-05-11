use std::io::{Read, BufReader, BufRead};

pub struct Input<R: Read> {
    inner: BufReader<R>
}

impl<R: Read> Input<R> {
    pub fn new(input: R) -> Self {
        Self {
            inner: BufReader::new(input)
        }
    }
    
    pub fn read_line(&mut self) -> std::io::Result<String> {
        let mut buf = String::new();
        self.inner.read_line(&mut buf)?;
        Ok(buf)
    }
}