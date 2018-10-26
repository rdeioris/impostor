extern crate clap;
extern crate impostor;

use clap::{App, Arg};

use impostor::adapter::BusAdapter;
use impostor::audio::Piano;
use impostor::memcontroller::MemoryControllerSmart;
use impostor::mos6502::MOS6502;
use impostor::ram::Ram;
use impostor::random::Random;
use impostor::rom::Rom;
use impostor::unixterm::UnixTerm;

use impostor::utils::to_number;
use impostor::Clock;

use impostor::graphics::vga_mode13h_palette::MODE13H_PALETTE;
use impostor::graphics::{Framebuffer, Screen, WindowEvent};
use impostor::input::{ElementState, VirtualKeyCode};

use impostor::AddressBusIO;
use impostor::Interrupt;

use std::cell::RefCell;
use std::fs;
use std::rc::Rc;

struct Sprite {
    pixels: [u8; 64],
    x: u8,
    y: u8,
    flags: u8,
    framebuffer: Framebuffer,
}

impl Sprite {
    fn new() -> Sprite {
        Sprite {
            pixels: [0; 64],
            x: 0,
            y: 0,
            flags: 0,
            framebuffer: Framebuffer::new(8, 8),
        }
    }
}

struct AivFrameBuffer {
    framebuffer: Framebuffer,
    screen: Screen,
    current_row: u8,
    current_col: u8,
    scroll_x: u8,
    scroll_y: u8,
    input: u8,
    sprites: [Sprite; 32],
}

impl AivFrameBuffer {
    fn new(screen: Screen, framebuffer: Framebuffer) -> AivFrameBuffer {
        AivFrameBuffer {
            screen,
            framebuffer,
            current_row: 0,
            current_col: 0,
            scroll_x: 0,
            scroll_y: 0,
            input: 0,
            sprites: [
                Sprite::new(),
                Sprite::new(),
                Sprite::new(),
                Sprite::new(),
                Sprite::new(),
                Sprite::new(),
                Sprite::new(),
                Sprite::new(),
                Sprite::new(),
                Sprite::new(),
                Sprite::new(),
                Sprite::new(),
                Sprite::new(),
                Sprite::new(),
                Sprite::new(),
                Sprite::new(),
                Sprite::new(),
                Sprite::new(),
                Sprite::new(),
                Sprite::new(),
                Sprite::new(),
                Sprite::new(),
                Sprite::new(),
                Sprite::new(),
                Sprite::new(),
                Sprite::new(),
                Sprite::new(),
                Sprite::new(),
                Sprite::new(),
                Sprite::new(),
                Sprite::new(),
                Sprite::new(),
            ],
        }
    }

    fn vblank(&mut self) -> bool {
        let mut input_state = 0;
        let mut exit = false;
        let delta = self.screen.width / self.framebuffer.width;
        let x = self.scroll_x as usize * delta;
        let y = self.scroll_y as usize * delta;
        self.screen.clear();
        self.framebuffer
            .blit(&self.screen, x, y, self.screen.width, self.screen.height);
        // draw sprites
        for i in 0..32 {
            let sprite = &mut self.sprites[i];
            if sprite.flags & 0x01 != 1 {
                continue;
            }
            for y in 0..8 {
                for x in 0..8 {
                    let sprite_x = if sprite.flags & 0x02 != 0 { 7 - x } else { x };
                    let sprite_y = if sprite.flags & 0x04 != 0 { 7 - y } else { y };
                    let color_address = sprite_y as usize * 8 + sprite_x as usize;
                    let color = sprite.pixels[color_address];
                    let pixel_address = (y * 8 * 3) + (x * 3);
                    let final_color = MODE13H_PALETTE[color as usize];
                    let pixels = &mut sprite.framebuffer.pixels;
                    pixels[pixel_address] = (final_color >> 16) as u8;
                    pixels[pixel_address + 1] = ((final_color >> 8) & 0xff) as u8;
                    pixels[pixel_address + 2] = (final_color & 0xff) as u8;
                }
            }
            let zoom = ((sprite.flags >> 4) + 1) as usize;
            sprite.framebuffer.blit(
                &self.screen,
                sprite.x as usize * delta,
                sprite.y as usize * delta,
                8 * delta * zoom,
                8 * delta * zoom,
            );
        }
        self.screen.swap();
        self.screen.poll_events(|event| match event {
            WindowEvent::CloseRequested => exit = true,
            WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
                Some(VirtualKeyCode::Escape) => exit = true,
                Some(VirtualKeyCode::Up) => {
                    if input.state == ElementState::Pressed {
                        input_state |= 0x01;
                    }
                }
                Some(VirtualKeyCode::Down) => {
                    if input.state == ElementState::Pressed {
                        input_state |= 0x02;
                    }
                }
                Some(VirtualKeyCode::Right) => {
                    if input.state == ElementState::Pressed {
                        input_state |= 0x04;
                    }
                }
                Some(VirtualKeyCode::Left) => {
                    if input.state == ElementState::Pressed {
                        input_state |= 0x08;
                    }
                }
                Some(VirtualKeyCode::Space) => {
                    if input.state == ElementState::Pressed {
                        input_state |= 0x10;
                    }
                }
                Some(VirtualKeyCode::LShift) => {
                    if input.state == ElementState::Pressed {
                        input_state |= 0x20;
                    }
                }
                Some(VirtualKeyCode::RShift) => {
                    if input.state == ElementState::Pressed {
                        input_state |= 0x40;
                    }
                }
                Some(VirtualKeyCode::LAlt) => {
                    if input.state == ElementState::Pressed {
                        input_state |= 0x80;
                    }
                }
                _ => (),
            },
            _ => (),
        });

        self.input = input_state;
        return exit;
    }
}

impl AddressBusIO<u16, u8> for AivFrameBuffer {
    fn write(&mut self, address: u16, value: u8) {
        if address < 256 {
            let pixel_address: usize =
                (self.current_row as usize * self.framebuffer.width * 3) + (address as usize * 3);
            let color = MODE13H_PALETTE[value as usize];
            self.framebuffer.pixels[pixel_address] = (color >> 16) as u8;
            self.framebuffer.pixels[pixel_address + 1] = ((color >> 8) & 0xff) as u8;
            self.framebuffer.pixels[pixel_address + 2] = (color & 0xff) as u8;
            return;
        }
        if address >= 0x200 && address < 0x4000 {
            let sprite_index = (address / 256) - 2;
            let sprite_base = address - (sprite_index as u16 + 2) * 256;
            let sprite = &mut self.sprites[sprite_index as usize];
            if sprite_base < 64 {
                sprite.pixels[sprite_base as usize] = value;
                return;
            }
            match sprite_base {
                64 => sprite.x = value,
                65 => sprite.y = value,
                66 => sprite.flags = value,
                _ => (),
            }
            return;
        }
        match address {
            256 => self.current_row = value,
            257 => {
                for y in 0..self.framebuffer.height {
                    for x in 0..self.framebuffer.width {
                        let pixel_address: usize = (y * self.framebuffer.width * 3) + (x * 3);
                        let color = MODE13H_PALETTE[value as usize];
                        self.framebuffer.pixels[pixel_address] = (color >> 16) as u8;
                        self.framebuffer.pixels[pixel_address + 1] = ((color >> 8) & 0xff) as u8;
                        self.framebuffer.pixels[pixel_address + 2] = (color & 0xff) as u8;
                    }
                }
            }
            258 => self.scroll_x = value,
            259 => self.scroll_y = value,
            260 => self.current_col = value,
            261 => {
                let pixel_address: usize = (self.current_row as usize * self.framebuffer.width * 3)
                    + (self.current_col as usize * 3);
                let color = MODE13H_PALETTE[value as usize];
                self.framebuffer.pixels[pixel_address] = (color >> 16) as u8;
                self.framebuffer.pixels[pixel_address + 1] = ((color >> 8) & 0xff) as u8;
                self.framebuffer.pixels[pixel_address + 2] = (color & 0xff) as u8;
            }
            _ => (),
        }
    }

    fn read(&mut self, address: u16) -> u8 {
        let mut value = 0;
        if address >= 0x200 && address < 0x4000 {
            let sprite_index = (address / 256) - 2;
            let sprite_base = address - (sprite_index as u16 + 2) * 256;
            let sprite = &self.sprites[sprite_index as usize];
            match sprite_base {
                64 => value = sprite.x,
                65 => value = sprite.y,
                66 => value = sprite.flags,
                _ => (),
            }
            return value;
        }

        match address {
            256 => value = self.current_row,
            258 => value = self.scroll_x,
            259 => value = self.scroll_y,
            260 => value = self.current_col,
            262 => value = self.input,
            _ => (),
        }
        return value;
    }
}

fn main() {
    let matches = App::new("aivmachine")
        .version("0.1")
        .author("Roberto De Ioris <roberto@aiv01.it>")
        .about("Didactical Fantasy Console")
        .arg(
            Arg::with_name("debug")
                .long("debug")
                .help("report CPU state after each opcode"),
        ).arg(
            Arg::with_name("no-vblank")
                .long("no-vblank")
                .help("disable NMI interrupt on vblank"),
        ).arg(
            Arg::with_name("pc")
                .required(false)
                .long("pc")
                .takes_value(true)
                .value_name("address")
                .help("set initial PC")
                .default_value("0xc000"),
        ).arg(
            Arg::with_name("hz")
                .required(false)
                .long("hz")
                .takes_value(true)
                .value_name("ticks")
                .help("set ticks per second")
                .default_value("1000000"),
        ).arg(
            Arg::with_name("piano-speed")
                .required(false)
                .long("piano-speed")
                .takes_value(true)
                .value_name("milliseconds")
                .help("set duration of a piano note")
                .default_value("125"),
        ).arg(Arg::with_name("romfile").index(1).required(true))
        .get_matches();

    let romfile = matches.value_of("romfile").unwrap();

    let pc: u16 = match to_number(matches.value_of("pc").unwrap()) {
        Ok(value) => value,
        Err(_) => panic!("invalid address format for pc"),
    };

    let hz: u32 = match to_number(matches.value_of("hz").unwrap()) {
        Ok(value) => value,
        Err(_) => panic!("invalid number format for hz"),
    };

    let piano_speed: u64 = match to_number(matches.value_of("piano-speed").unwrap()) {
        Ok(value) => value,
        Err(_) => panic!("invalid number format for piano-speed"),
    };

    let mut rom = Rom::new(fs::read(romfile).unwrap());

    let mut ram = Ram::new(4096);

    let mut term8 = UnixTerm::new();

    let mut term = BusAdapter::new(&mut term8);

    let mut piano = Piano::new(piano_speed);

    let mut random = Random::new();

    let screen = Screen::new("aivmachine", 512, 512);

    let framebuffer = Framebuffer::new(256, 256);

    let aiv_framebuffer = Rc::new(RefCell::new(AivFrameBuffer::new(screen, framebuffer)));

    let mut memory_controller = MemoryControllerSmart::new();
    memory_controller.map(0x0000, 0x1fff, &mut ram);
    memory_controller.map(0x2000, 0x2003, &mut term);

    memory_controller.map(0x2004, 0x2004, &mut piano);

    memory_controller.map(0x2005, 0x2005, &mut random);

    let borrowed_aiv_framebuffer = Rc::clone(&aiv_framebuffer);
    memory_controller.map_shared(0x4000, 0x7fff, borrowed_aiv_framebuffer);

    memory_controller.map(0xc000, 0xffff, &mut rom);

    let mut cpu = MOS6502::new(memory_controller);
    cpu.pc = pc;
    cpu.debug = matches.is_present("debug");

    let block_nmi = matches.is_present("no-vblank");

    let mut last_ticks: u64 = 0;

    let ticks_per_frame = hz / 60;

    loop {
        let mut ticks_counter = ticks_per_frame as i64;
        while ticks_counter > 0 {
            cpu.step();
            if cpu.debug {
                println!("[{:04X}] {}", cpu.debug_pc, cpu.debug_line);
            }
            ticks_counter -= (cpu.ticks - last_ticks) as i64;
            last_ticks = cpu.ticks;
        }
        if aiv_framebuffer.borrow_mut().vblank() {
            break;
        }
        // avoid NMI if the related vector is not in the rom
        if !block_nmi && cpu.read(0xfffb) >= 0xc0 {
            cpu.raise(6);
        }
    }
}
