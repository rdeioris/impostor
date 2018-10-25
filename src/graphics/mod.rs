extern crate gl;
extern crate glutin;

use std::mem;

use graphics::glutin::GlContext;

pub use graphics::glutin::{ElementState, VirtualKeyCode, WindowEvent};
pub mod vga_mode13h_palette;

pub struct Screen {
    pub width: usize,
    pub height: usize,
    pub event_loop: glutin::EventsLoop,
    pub gl_window: glutin::GlWindow,
}

impl Screen {
    pub fn new(title: &'static str, width: usize, height: usize) -> Screen {
        let logical_size = glutin::dpi::LogicalSize::new(width as f64, height as f64);

        let window = glutin::WindowBuilder::new()
            .with_title(title)
            .with_dimensions(logical_size);

        let context = glutin::ContextBuilder::new().with_vsync(true);

        let event_loop = glutin::EventsLoop::new();

        let gl_window = glutin::GlWindow::new(window, context, &event_loop).unwrap();

        let screen = Screen {
            width,
            height,
            event_loop,
            gl_window,
        };

        unsafe {
            screen.gl_window.make_current().unwrap();
            gl::load_with(|symbol| screen.gl_window.get_proc_address(symbol) as *const _);
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        }

        return screen;
    }

    pub fn swap(&self) {
        self.gl_window.swap_buffers().unwrap();
    }

    pub fn poll_events<F: FnMut(glutin::WindowEvent)>(&mut self, mut callback: F) {
        self.event_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => callback(event),
            _ => (),
        });
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
                x as i32,
                (screen.height - y) as i32,
                (x + width) as i32,
                (screen.height - y - height) as i32,
                gl::COLOR_BUFFER_BIT,
                gl::NEAREST,
            );
        }
    }
}
