extern crate gl;
extern crate glutin;

use std::mem;

pub use self::glutin::{ElementState, VirtualKeyCode, WindowEvent};
pub mod vga_mode13h_palette;
use self::glutin::dpi::{LogicalSize, PhysicalSize};

pub struct Screen {
    pub width: usize,
    pub height: usize,
    pub event_loop: glutin::EventsLoop,
    window_context: glutin::WindowedContext<glutin::PossiblyCurrent>,
    logical_size: LogicalSize,
}

impl Screen {
    pub fn new(title: &'static str, width: usize, height: usize) -> Screen {
        let logical_size = LogicalSize::new(width as f64, height as f64);

        let window_builder = glutin::WindowBuilder::new()
            .with_title(title)
            .with_resizable(false)
            .with_dimensions(logical_size);

        let event_loop = glutin::EventsLoop::new();

        let context = glutin::ContextBuilder::new()
            .with_vsync(true)
            .build_windowed(window_builder, &event_loop)
            .unwrap();
        let window_context = unsafe { context.make_current().unwrap() };
        unsafe {
            gl::load_with(|symbol| window_context.get_proc_address(symbol) as *const _);
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        }

        let screen = Screen {
            width,
            height,
            event_loop,
            window_context,
            logical_size,
        };

        return screen;
    }

    pub fn swap(&self) {
        self.window_context.swap_buffers().unwrap();
    }

    pub fn poll_events<F: FnMut(glutin::WindowEvent)>(&mut self, mut callback: F) {
        self.event_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => callback(event),
            _ => (),
        });
        // Fix for macOS Mojave that has rendering issues
        if cfg!(target_os = "macos") {
            self.window_context.resize(PhysicalSize::from_logical(
                self.logical_size,
                self.window_context.window().get_hidpi_factor(),
            ));
        }
    }

    pub fn clear(&self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }
}

pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    texture: gl::types::GLuint,
    framebuffer: gl::types::GLuint,
    pub pixels: Vec<u8>,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Framebuffer {
        let pixels = vec![0; width * height * 3];

        unsafe {
            let mut texture = mem::uninitialized();
            let mut framebuffer = mem::uninitialized();
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB as i32,
                width as i32,
                height as i32,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                pixels.as_ptr() as *const _,
            );

            gl::GenFramebuffers(1, &mut framebuffer);
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, framebuffer);
            gl::FramebufferTexture2D(
                gl::READ_FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                texture,
                0,
            );
            Framebuffer {
                width,
                height,
                texture,
                framebuffer,
                pixels,
            }
        }
    }

    pub fn blit(&self, screen: &Screen, x: usize, y: usize, width: usize, height: usize) {
        // required for correct size when monitor dpi changes
        let dpi_factor = screen.window_context.window().get_hidpi_factor();
        let physical_size = PhysicalSize::from_logical(screen.logical_size, dpi_factor);
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB as i32,
                self.width as i32,
                self.height as i32,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                self.pixels.as_ptr() as *const _,
            );
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, self.framebuffer);
            gl::BlitFramebuffer(
                0,
                0,
                self.width as i32,
                self.height as i32,
                (x as f64 * dpi_factor) as i32,
                (physical_size.height - (y as f64 * dpi_factor)) as i32,
                ((x + width) as f64 * dpi_factor) as i32,
                (physical_size.height - ((y + height) as f64 * dpi_factor)) as i32,
                gl::COLOR_BUFFER_BIT,
                gl::NEAREST,
            );
        }
    }
}
