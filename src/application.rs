use std::io::Write;
use sysinfo::SystemExt;

use crate::mandelbrot_renderer::MandelbrotRenderer;
use crate::input::Input;
use crate::output::Output;
use crate::grayscale_encoder::GrayscaleEncoder;
use crate::render_program::RenderProgram;
use crate::progress_bar::ProgressBar;

type GenericResult<R> = Result<R, Box<dyn std::error::Error>>;

pub struct Application<W: Write> {
    args: Vec<String>,
    input: Input<std::io::Stdin>,
    output: Output<std::io::Stdout>,
    w: W,
    system: sysinfo::System,
    bounds: (usize, usize),
    remaining_memory_in_bytes: usize,
    threads_to_use: usize,
    iteration_count: usize
}

impl<W: Write> Application<W> {
    const DEFAULT_BOUNDS: (usize, usize) = (640, 480);
    const KILO: usize = 1024;
    const GIGA: usize = 1024 * 1024 * 1024;
    const DEFAULT_PROGRESS_BAR_LENGTH: usize = 30;

    pub fn new(args: Vec<String>, w: W) -> Self {
        Self {
            args,
            input: Input::new(std::io::stdin()),
            output: Output::new(std::io::stdout()),
            w,
            system: sysinfo::System::new_with_specifics(sysinfo::RefreshKind::new().with_memory()),
            bounds: (0, 0),
            remaining_memory_in_bytes: 0,
            threads_to_use: 0,
            iteration_count: 0
        }
    }

    pub fn run(&mut self) {
        if !self.are_args_valid() {
            self.print_usage();
            return;
        }

        self.parse_args();
        self.print_rendering_info();

        if !self.should_continue() {
            return;
        }

        let buf_size = self.calculate_buf_size();
        let renderer = MandelbrotRenderer::new(self.bounds, MandelbrotRenderer::DEFAULT_PLANE_BOUNDS, self.iteration_count);
        let mut render_program = RenderProgram::new(renderer, buf_size, self.threads_to_use);

        let mut progress_bar = ProgressBar::new('=', Self::DEFAULT_PROGRESS_BAR_LENGTH, Output::new(std::io::stdout()));
        progress_bar.print_progress(0.0);
        render_program.set_progress_callback(move |done| progress_bar.print_progress(done));

        let mut writer = GrayscaleEncoder::new(&mut self.w, self.bounds).write_header().unwrap();
        let stream_writer = writer.stream_writer_with_size(buf_size).unwrap();
        let render_info = render_program.begin(stream_writer);

        self.output.write_line_flushed(&format!("\n{} bytes rendered in {} minutes {} seconds {} milliseconds",
            render_info.bytes,
            render_info.elapsed.as_secs() / 60,
            render_info.elapsed.as_secs() % 60,
            render_info.elapsed.as_millis() % 1000));
    }

    fn are_args_valid(&mut self) -> bool {
        self.args.len() == 6
    }

    fn print_usage(&mut self) {
        self.output.write_line_flushed("Usage: [width] [height] [remaining memory in GB] [number threads to use] [number of iterations]");
    }

    fn parse_args(&mut self) {
        self.bounds = self.parse_resolution().unwrap_or_else(|err| {
            self.output.write_line_flushed(&format!("Failed to parse resolution: {}\nSwitching to default", err));
            Self::DEFAULT_BOUNDS
        });

        self.remaining_memory_in_bytes = Self::GIGA.max(self.parse_remaining_memory().unwrap_or_else(|err| {
            self.output.write_line_flushed(&format!("Failed to parse remaining memory: {}\nAt least 1GB of memory will be free", err));
            Self::GIGA
        }));

        self.threads_to_use = self.parse_number_of_threads_to_use().unwrap_or_else(|err| {
            self.output.write_line_flushed(&format!("Failed to parse number of threads to use: {}\nUsing 1 thread", err));
            1
        }).clamp(1, usize::from(std::thread::available_parallelism().unwrap()));

        self.iteration_count = self.parse_number_of_iterations().unwrap_or_else(|err| {
            self.output.write_line_flushed(&format!("Failed to parse number of iterations: {}\nUsing 100 iterations to render each pixel", err));
            100
        });
    }

    fn parse_resolution(&self) -> GenericResult<(usize, usize)> {
        Ok((self.args[1].parse::<usize>()?, self.args[2].parse::<usize>()?))
    }

    fn parse_remaining_memory(&self) -> GenericResult<usize> {
        Ok(self.args[3].parse::<usize>()? * Self::GIGA)
    }

    fn parse_number_of_threads_to_use(&self) -> GenericResult<usize> {
        Ok(self.args[4].parse::<usize>()?)
    }

    fn parse_number_of_iterations(&self) -> GenericResult<usize> {
        Ok(self.args[5].parse::<usize>()?)
    }

    fn print_rendering_info(&mut self) {
        self.output.write_line_flushed(&format!("Resolution: {}x{}\nRemaining memory: {:.1}GB\nThreads used: {}\nNumber of iterations to render each pixel: {}",
            self.bounds.0, self.bounds.1,
            self.remaining_memory_in_bytes as f32 / Self::GIGA as f32,
            self.threads_to_use,
            self.iteration_count));
    }

    fn should_continue(&mut self) -> bool {
        self.output.write_text_flushed("Continue?[Y/n]: ");

        let answer = self.input.read_line().unwrap_or("n".into()).trim().to_ascii_lowercase();
        answer.len() > 1 || answer.as_bytes()[0] == b'y'
    }

    fn calculate_buf_size(&self) -> usize {
        let available_memory = self.system.available_memory() as usize * Self::KILO;
        let (memory_to_use, image_size) = (available_memory - self.remaining_memory_in_bytes, self.bounds.0 * self.bounds.1);
        if memory_to_use / 3 > image_size { image_size } else { memory_to_use / 3 }
    }
}