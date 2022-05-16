extern crate crossbeam;
extern crate sysinfo;
extern crate png;

mod complex;
mod mandelbrot_renderer;
mod application;
mod input;
mod output;
mod render_program;
mod progress_bar;
mod render_info;
mod grayscale_encoder;

use std::io::{BufReader, BufWriter};
use application::Application;

fn main() {
    let (stdin, stdout) = (std::io::stdin(), std::io::stdout());
    let (input, output) = (BufReader::new(stdin.lock()), BufWriter::new(stdout));
    let file = BufWriter::new(std::fs::File::create("mandelbrot.png").unwrap());
    let mut app = Application::new(std::env::args().collect::<Vec<String>>(), input, output, file);
    app.run();
}