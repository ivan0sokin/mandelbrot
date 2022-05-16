use std::sync::{Arc, Mutex};
use std::io::Write;
use std::sync::atomic::Ordering;

use crate::mandelbrot_renderer::MandelbrotRenderer;
use crate::render_info::RenderInfo;

pub struct RenderProgram<'a> {
    renderer: MandelbrotRenderer,
    buf: Vec<u8>,
    threads_to_use: usize,
    progress_callback: Box<dyn FnMut(f64) + Send + Sync + 'a>
}

impl<'a> RenderProgram<'a> {
    pub fn new(renderer: MandelbrotRenderer, buf_size: usize, threads_to_use: usize) -> Self {
        Self {
            renderer,
            buf: vec![0; buf_size],
            threads_to_use,
            progress_callback: Box::new(|_| {})
        }
    }

    pub fn set_progress_callback(&mut self, progress_callback: impl FnMut(f64) + Send + Sync + 'a) {
        self.progress_callback = Box::new(progress_callback);
    }

    pub fn begin<W: Write>(&mut self, mut writer: W) -> RenderInfo {
        let bytes_to_render = self.renderer.get_canvas_resolution();
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
        self.renderer.render(&mut self.buf, buf_offset);

        let bytes_done = buf_offset + self.buf.len();
        (self.progress_callback)(bytes_done as f64 / self.renderer.get_canvas_resolution() as f64);
    }

    fn run_multithreaded(&mut self, buf_offset: usize) {
        let bytes_per_thread = self.buf.len() / self.threads_to_use + 1;
        let chunks = self.buf.chunks_mut(bytes_per_thread).collect::<Vec<&mut [u8]>>();
        
        let canvas_resolution = self.renderer.get_canvas_resolution() as f64;
        
        let bytes_done = Arc::new(std::sync::atomic::AtomicUsize::new(buf_offset));
        let progress_callback = Arc::new(Mutex::new(&mut self.progress_callback));
        
        crossbeam::scope(|scope| {
            for (i, chunk) in chunks.into_iter().enumerate() {
                let offset = buf_offset + i * bytes_per_thread;
                let thread_renderer = self.renderer.clone();
                let bytes_done_clone = Arc::clone(&bytes_done);
                let progress_callback_clone = Arc::clone(&progress_callback);
                scope.spawn(move |_| {
                    thread_renderer.render(chunk, offset);
                    bytes_done_clone.fetch_add(chunk.len(), Ordering::SeqCst);
                    (progress_callback_clone.lock().unwrap())(bytes_done_clone.load(Ordering::Relaxed) as f64 / canvas_resolution);
                });
            }
        }).unwrap();
    }
}