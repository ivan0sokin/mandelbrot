use std::sync::{Arc, Mutex};
use std::io::Write;
use std::sync::atomic::Ordering;

use crate::mandelbrot_renderer::{FloatType, MandelbrotRenderer};
use crate::render_info::RenderInfo;
use crate::complex::Complex;
use crate::color::{RGB, Gray};


pub struct RenderProgram<'a> {
    renderer: MandelbrotRenderer,
    buf: Vec<u8>,
    threads_to_use: usize,
    progress_callback: Box<dyn FnMut(f64) + Send + 'a>,
    get_color: Box<dyn Fn(&Complex<FloatType>, Option<usize>) -> RGB<FloatType> + Send + 'a>
}

impl<'a> RenderProgram<'a> {
    pub fn new(renderer: MandelbrotRenderer, buf_size: usize, threads_to_use: usize) -> Self {
        Self {
            renderer,
            buf: vec![0; buf_size],
            threads_to_use,
            progress_callback: Box::new(|_| {}),
            get_color: Box::new(|_, _| { RGB::default()})
        }
    }

    pub fn set_progress_callback(&mut self, progress_callback: impl FnMut(f64) + Send + 'a) {
        self.progress_callback = Box::new(progress_callback);
    }

    pub fn color_pixel(&mut self, get_color: impl Fn(&Complex<FloatType>, Option<usize>) -> RGB<FloatType> + Send + 'a) {
        self.get_color = Box::new(get_color);
    }

    pub fn begin<W: Write>(&mut self, mut writer: W) -> RenderInfo {
        let bytes_to_render = self.renderer.get_canvas_resolution() * 3;
        let mut rendering_time = std::time::Duration::default();

        (self.progress_callback)(0.0);

        let mut buf_offset = 0;
        while buf_offset < bytes_to_render {
            if bytes_to_render - buf_offset < self.buf.len() {
                self.buf.resize(bytes_to_render - buf_offset, 0);
            }

            let now = std::time::Instant::now();

            if self.threads_to_use == 1 {
                self.run_singlethreaded(buf_offset);
            } else {
                self.run_multithreaded(buf_offset);
            }

            rendering_time += now.elapsed();

            writer.write_all(&self.buf).unwrap();
            buf_offset += self.buf.len();
        }

        writer.flush().unwrap();

        RenderInfo::new(rendering_time, bytes_to_render)
    }

    fn run_singlethreaded(&mut self, buf_offset: usize) {
        self.renderer.render(&mut self.buf, buf_offset, |z, escape_time| (self.get_color)(z, escape_time));

        let bytes_done = buf_offset + self.buf.len();
        (self.progress_callback)(bytes_done as f64 / (self.renderer.get_canvas_resolution() as f64 * 3.0));
    }

    fn run_multithreaded(&mut self, buf_offset: usize) {
        let bytes_per_thread = self.closest_number_to_three_over(self.buf.len() / self.threads_to_use + 1);
        let chunks = self.buf.chunks_mut(bytes_per_thread).collect::<Vec<&mut [u8]>>();
        
        let canvas_resolution = self.renderer.get_canvas_resolution() as f64 * 3.0;
        
        let bytes_done = Arc::new(std::sync::atomic::AtomicUsize::new(buf_offset));
        let progress_callback = Arc::new(Mutex::new(&mut self.progress_callback));
        let get_color = Arc::new(Mutex::new(&mut self.get_color));

        crossbeam::scope(|scope| {
            for (i, chunk) in chunks.into_iter().enumerate() {
                let offset = buf_offset + i * bytes_per_thread;
                let thread_renderer = self.renderer.clone();
                let bytes_done_clone = Arc::clone(&bytes_done);
                let progress_callback_clone = Arc::clone(&progress_callback);
                let get_color_clone = Arc::clone(&get_color);
                scope.spawn(move |_| {
                    thread_renderer.render(chunk, offset, |z, escape_time| (get_color_clone.lock().unwrap())(z, escape_time));
                    bytes_done_clone.fetch_add(chunk.len(), Ordering::SeqCst);
                    (progress_callback_clone.lock().unwrap())(bytes_done_clone.load(Ordering::Relaxed) as f64 / canvas_resolution);
                });
            }
        }).unwrap();
    }

    fn closest_number_to_three_over(&self, n: usize) -> usize {
        match n % 3 {
            0 => n,
            1 => n + 2,
            2 => n + 1,
            _ => 0
        }
    }
}