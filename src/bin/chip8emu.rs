extern crate impostor;

use impostor::audio::Beeper;
use impostor::chip8::Chip8;
use impostor::graphics::{Framebuffer, Screen, WindowEvent};
use impostor::input::{ElementState, VirtualKeyCode};
use impostor::ram::Ram;

use impostor::Clock;

use std::env;
use std::fs;

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
        0xF0, 0x80, 0xF0, 0x80, 0x80, // F
    ];

    ram.fill(fonts.to_vec(), 0x000);

    ram.fill(fs::read(&*args[1]).unwrap(), 0x200);

    let mut chip8 = Chip8::new(ram);

    let mut screen = Screen::new("chip8", 1024, 512);

    let mut framebuffer = Framebuffer::new(64, 32);

    let beeper = Beeper::new(880);

    let mut running = true;
    while running {
        screen.poll_events(|event| match event {
            WindowEvent::CloseRequested => running = false,
            WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
                Some(VirtualKeyCode::Escape) => running = false,
                Some(VirtualKeyCode::Key0) => {
                    chip8.keys[0x0] = input.state == ElementState::Pressed
                }
                Some(VirtualKeyCode::Key1) => {
                    chip8.keys[0x1] = input.state == ElementState::Pressed
                }
                Some(VirtualKeyCode::Key2) => {
                    chip8.keys[0x2] = input.state == ElementState::Pressed
                }
                Some(VirtualKeyCode::Key3) => {
                    chip8.keys[0x3] = input.state == ElementState::Pressed
                }
                Some(VirtualKeyCode::Key4) => {
                    chip8.keys[0x4] = input.state == ElementState::Pressed
                }
                Some(VirtualKeyCode::Key5) => {
                    chip8.keys[0x5] = input.state == ElementState::Pressed
                }
                Some(VirtualKeyCode::Key6) => {
                    chip8.keys[0x6] = input.state == ElementState::Pressed
                }
                Some(VirtualKeyCode::Key7) => {
                    chip8.keys[0x7] = input.state == ElementState::Pressed
                }
                Some(VirtualKeyCode::Key8) => {
                    chip8.keys[0x8] = input.state == ElementState::Pressed
                }
                Some(VirtualKeyCode::Key9) => {
                    chip8.keys[0x9] = input.state == ElementState::Pressed
                }
                Some(VirtualKeyCode::A) => chip8.keys[0xa] = input.state == ElementState::Pressed,
                Some(VirtualKeyCode::B) => chip8.keys[0xb] = input.state == ElementState::Pressed,
                Some(VirtualKeyCode::C) => chip8.keys[0xc] = input.state == ElementState::Pressed,
                Some(VirtualKeyCode::D) => chip8.keys[0xd] = input.state == ElementState::Pressed,
                Some(VirtualKeyCode::E) => chip8.keys[0xe] = input.state == ElementState::Pressed,
                Some(VirtualKeyCode::F) => chip8.keys[0xf] = input.state == ElementState::Pressed,
                _ => (),
            },
            _ => (),
        });
        chip8.step();

        if chip8.delay_timer > 0 {
            chip8.delay_timer -= 1;
        }

        if chip8.sound_timer > 0 {
            chip8.sound_timer -= 1;
            if chip8.sound_timer == 0 {
                beeper.beep();
            }
        }

        if chip8.redraw {
            for y in 0..32 {
                for x in 0..64 {
                    let screen_offset = y * 64 + x;
                    let pixels_offset = (y * 64 + x) * 3;
                    framebuffer.pixels[pixels_offset] = chip8.screen[screen_offset] * 255;
                    framebuffer.pixels[pixels_offset + 1] = framebuffer.pixels[pixels_offset];
                    framebuffer.pixels[pixels_offset + 2] = framebuffer.pixels[pixels_offset];
                }
            }
            framebuffer.blit(0, 0, screen.width, screen.height);
            screen.swap();
            chip8.redraw = false;
        }
    }
}
