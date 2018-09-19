extern crate gl;
extern crate glutin;
extern crate impostor;

use impostor::chip8::Chip8;
use impostor::ram::Ram;

use impostor::Clock;

use std::env;
use std::fs;

use std::mem;


use glutin::dpi::*;
use glutin::GlContext;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut ram = Ram::new(4096);

    let fonts = [
  0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
  0x20, 0x60, 0x20, 0x20, 0x70, // 1
  0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
  0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
  0x90, 0x90, 0xF0, 0x10, 0x10, // 4
  0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
  0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
  0xF0, 0x10, 0x20, 0x40, 0x40, // 7
  0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
  0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
  0xF0, 0x90, 0xF0, 0x90, 0x90, // A
  0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
  0xF0, 0x80, 0x80, 0x80, 0xF0, // C
  0xE0, 0x90, 0x90, 0x90, 0xE0, // D
  0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
  0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];


    ram.fill(fonts.to_vec(), 0x000);

    ram.fill(fs::read(&*args[1]).unwrap(), 0x200);

    let mut events_loop = glutin::EventsLoop::new();

    let mut chip8 = Chip8::new(ram);

    let window = glutin::WindowBuilder::new()
        .with_title("Hello, world!")
        .with_dimensions(LogicalSize::new(1024.0, 512.0));

    let context = glutin::ContextBuilder::new()
        .with_vsync(true);

    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();

    let mut pixels = [0; 64 * 32 * 3];

    unsafe {
        gl_window.make_current().unwrap();
    }

    unsafe {
        gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);

        let mut screen_texture = mem::uninitialized();
        gl::GenTextures(1, &mut screen_texture);
        gl::BindTexture(gl::TEXTURE_2D, screen_texture);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, 64, 32, 0, gl::RGB, gl::UNSIGNED_BYTE, pixels.as_ptr() as *const _);

        let mut framebuffer = mem::uninitialized();
        gl::GenFramebuffers(1, &mut framebuffer);
        gl::BindFramebuffer(gl::READ_FRAMEBUFFER, framebuffer);
        gl::FramebufferTexture2D(gl::READ_FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, screen_texture, 0);
    }

    let mut running = true;
    while running {

        events_loop.poll_events(|_event| {
        });

        chip8.step();


        if chip8.delay_timer > 0 {
            chip8.delay_timer -= 1;
        }

        if chip8.sound_timer > 0 {
            chip8.sound_timer -= 1;
            if chip8.sound_timer == 0 {
                println!("BEEEEEP");
            }
        }

        for y in 0..32 {
            for x in 0..64 {
                let screen_offset = y * 64 + x;
                let pixels_offset = ((31 - y) * 64 + x) * 3;
                pixels[pixels_offset] = chip8.screen[screen_offset] * 255;
                pixels[pixels_offset+1] = pixels[pixels_offset];
                pixels[pixels_offset+2] = pixels[pixels_offset];
            }
        }

        unsafe {
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, 64, 32, 0, gl::RGB, gl::UNSIGNED_BYTE, pixels.as_ptr() as *const _);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::BlitFramebuffer(0, 0, 64, 32, 0, 0, 1024, 512, gl::COLOR_BUFFER_BIT, gl::NEAREST);
        }

        gl_window.swap_buffers().unwrap();
    }
}
