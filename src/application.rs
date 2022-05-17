use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use sysinfo::SystemExt;

use crate::mandelbrot_renderer::MandelbrotRenderer;
use crate::input::Input;
use crate::output::Output;
use crate::grayscale_encoder::GrayscaleEncoder;
use crate::rgb_encoder::RGBEncoder;
use crate::render_program::RenderProgram;
use crate::color::{RGB, Gray};
use crate::progress_bar::ProgressBar;

type GenericResult<R> = Result<R, Box<dyn std::error::Error>>;

pub struct Application<I: Read, O: Write, W: Write> {
    args: Vec<String>,
    input: Input<I>,
    output: Output<O>,
    writer: W,
    system: sysinfo::System,
    bounds: (usize, usize),
    remaining_memory_in_bytes: usize,
    threads_to_use: usize,
    iteration_count: usize
}

impl<I: Read, O: Write + Send, W: Write> Application<I, O, W> {
    const DEFAULT_BOUNDS: (usize, usize) = (640, 480);
    const KILO: usize = 1024;
    const GIGA: usize = 1024 * 1024 * 1024;
    const DEFAULT_PROGRESS_BAR_LENGTH: usize = 30;

    pub fn new(args: Vec<String>, input: I, output: O, writer: W) -> Self {
        Self {
            args,
            input: Input::new(input),
            output: Output::new(output),
            writer,
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
        let renderer = MandelbrotRenderer::new(self.bounds, MandelbrotRenderer::WIKI_PLANE_BOUNDS, self.iteration_count);
        let mut render_program = RenderProgram::new(renderer, buf_size, self.threads_to_use);

        let palette: [RGB<u8>; 16] = [
            RGB::new(66, 30, 15),
            RGB::new(25, 7, 26),
            RGB::new(9, 1, 47),
            RGB::new(4, 4, 73),
            RGB::new(0, 7, 100),
            RGB::new(12, 44, 138),
            RGB::new(24, 82, 177),
            RGB::new(57, 125, 209),
            RGB::new(134, 181, 229),
            RGB::new(211, 236, 248),
            RGB::new(241, 233, 191),
            RGB::new(248, 201, 95),
            RGB::new(255, 170, 0),
            RGB::new(204, 128, 0),
            RGB::new(153, 87, 0),
            RGB::new(106, 52, 3)
        ];

        render_program.color_pixel(move |z, escape_time| {
            if let Some(t) = escape_time {
                palette[t % 16].to_zero_to_one_interval_rgb()
            } else {
                RGB::from_scalar(0.0)
            }
        });

        let thread_safe_output = Arc::new(Mutex::new(&mut self.output));
        let output_copy = Arc::clone(&thread_safe_output);

        let mut progress_bar = ProgressBar::new('=', Self::DEFAULT_PROGRESS_BAR_LENGTH);
        render_program.set_progress_callback(move |done| {
            progress_bar.update(done);
            thread_safe_output.lock().unwrap().write_text_flushed(&format!("{}", progress_bar));
        });

        let mut writer = RGBEncoder::new(&mut self.writer, self.bounds).write_header().unwrap();
        let stream_writer = writer.stream_writer_with_size(buf_size).unwrap();
        let render_info = render_program.begin(stream_writer);

        output_copy.lock().unwrap().write_line_flushed(&format!("\n{} bytes rendered in {} minutes {} seconds {} milliseconds",
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
        let (memory_to_use, image_size) = (available_memory - self.remaining_memory_in_bytes, self.bounds.0 * self.bounds.1 * 3);
        if memory_to_use / 3 > image_size { image_size } else { self.closest_number_to_three_below(memory_to_use / 3) }
    }

    fn closest_number_to_three_below(&self, n: usize) -> usize {
        match n % 3 {
            0 => n,
            1 => n - 1,
            2 => n - 2,
            _ => 0
        }
    }
}