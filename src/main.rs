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

use std::io::BufWriter;
use application::Application;

fn main() {
    let file = BufWriter::new(std::fs::File::create("mandelbrot.png").unwrap());
    let mut app = Application::new(std::env::args().collect::<Vec<String>>(), file);
    app.run();
}