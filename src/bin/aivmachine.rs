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
use impostor::Debug;
use impostor::Interrupt;

use impostor::debugger::debugger;

use std::cell::RefCell;
use std::fs;
use std::rc::Rc;

#[derive(Copy, Clone)]
struct Sprite {
    tile: u8,
    x: u8,
    y: u8,
    flags: u8,
}

impl Sprite {
    fn new() -> Sprite {
        Sprite {
            tile: 0,
            x: 0,
            y: 0,
            flags: 0,
        }
    }
}

struct AivFrameBuffer {
    framebuffer: Framebuffer,
    screen: Screen,
    background_color: u8,
    current_row: u8,
    current_col: u8,
    scroll_x: u8,
    scroll_y: u8,
    input: u8,
    background0: [u8; 32 * 32],
    background1: [u8; 32 * 32],
    background2: [u8; 32 * 32],
    background3: [u8; 32 * 32],
    background_mode: u8,
    sprites: [Sprite; 64],
    chr_ram: [u8; 256 * 256],
}

impl AivFrameBuffer {
    fn new(screen: Screen, framebuffer: Framebuffer) -> AivFrameBuffer {
        let sprite = Sprite::new();
        AivFrameBuffer {
            screen,
            framebuffer,
            background_color: 0,
            current_row: 0,
            current_col: 0,
            scroll_x: 0,
            scroll_y: 0,
            input: 0,
            background0: [0; 32 * 32],
            background1: [0; 32 * 32],
            background2: [0; 32 * 32],
            background3: [0; 32 * 32],
            background_mode: 0,
            sprites: [sprite; 64],
            chr_ram: [0; 256 * 256],
        }
    }

    fn write_pixel(&mut self, x: u8, y: u8, color: u8) {
        let pixels = &mut self.framebuffer.pixels;
        let pixel_address = (y as usize * self.framebuffer.width * 3) + (x as usize * 3);
        let color_rgb = MODE13H_PALETTE[color as usize];
        pixels[pixel_address] = (color_rgb >> 16) as u8;
        pixels[pixel_address + 1] = ((color_rgb >> 8) & 0xff) as u8;
        pixels[pixel_address + 2] = (color_rgb & 0xff) as u8;
    }

    fn vblank(&mut self) -> bool {
        self.screen.clear();
        let background_tile_size = if self.background_mode & 0x01 == 1 {
            16
        } else {
            8
        };
        for y in 0..=255 {
            for x in 0..=255 {
                let mut background_block = self.background_mode >> 1 & 0x03;
                // set background color
                let background_color = self.background_color;
                self.write_pixel(x, y, background_color);
                // get background tile for the pixel
                let mut absolute_x = x as usize + self.scroll_x as usize;
                let mut absolute_y = y as usize + self.scroll_y as usize;
                if absolute_x > 255 {
                    match background_block {
                        0 => background_block = 1,
                        1 => background_block = 0,
                        2 => background_block = 3,
                        3 => background_block = 2,
                        _ => (),
                    }
                }
                if absolute_y > 255 {
                    match background_block {
                        0 => background_block = 2,
                        1 => background_block = 3,
                        2 => background_block = 0,
                        3 => background_block = 1,
                        _ => (),
                    }
                }

                absolute_x %= 256;
                absolute_y %= 256;

                let tile_x = absolute_x / background_tile_size;
                let tile_y = absolute_y / background_tile_size;
                let mut tile = 0;
                match background_block {
                    0 => tile = self.background0[tile_y * (256 / background_tile_size) + tile_x],
                    1 => tile = self.background1[tile_y * (256 / background_tile_size) + tile_x],
                    2 => tile = self.background2[tile_y * (256 / background_tile_size) + tile_x],
                    3 => tile = self.background3[tile_y * (256 / background_tile_size) + tile_x],
                    _ => (),
                }
                // get pixel tile
                let tile_pixel_x = absolute_x % background_tile_size;
                let tile_pixel_y = absolute_y % background_tile_size;
                let tile_chr_x = (tile as usize % (256 / background_tile_size))
                    * background_tile_size
                    + tile_pixel_x;
                let tile_chr_y = (tile as usize / (256 / background_tile_size))
                    * background_tile_size
                    + tile_pixel_y;
                let tile_address = tile_chr_y * 256 + tile_chr_x;
                let tile_pixel_color = self.chr_ram[tile_address];
                // write it in the framebuffer (if not 0)
                if tile_pixel_color != 0 {
                    self.write_pixel(x, y, tile_pixel_color);
                }
            }
        }

        // check each sprite
        for i in 0..=63 {
            let sprite = self.sprites[i];
            // check if the sprite is enabled
            if sprite.flags & 0x01 == 0 {
                continue;
            }
            let sprite_tile_size = if (sprite.flags >> 3) & 0x01 == 1 {
                16
            } else {
                8
            };
            let sprite_x = sprite.x as usize;
            let sprite_y = sprite.y as usize;
            let scale = ((sprite.flags >> 4) + 1) as usize;

            for y in 0..(sprite_tile_size * scale) {
                for x in 0..(sprite_tile_size * scale) {
                    let flipped_x = if (sprite.flags >> 1) & 0x01 == 0 {
                        x
                    } else {
                        ((sprite_tile_size * scale) - 1) - x
                    };
                    let flipped_y = if (sprite.flags >> 2) & 0x01 == 0 {
                        y
                    } else {
                        ((sprite_tile_size * scale) - 1) - y
                    };
                    let tile_pixel_x = flipped_x / scale;
                    let tile_pixel_y = flipped_y / scale;
                    let tile_chr_x = (sprite.tile as usize % (256 / sprite_tile_size))
                        * sprite_tile_size
                        + tile_pixel_x;
                    let tile_chr_y = (sprite.tile as usize / (256 / sprite_tile_size))
                        * sprite_tile_size
                        + tile_pixel_y;
                    let tile_address = tile_chr_y * 256 + tile_chr_x;
                    let tile_pixel_color = self.chr_ram[tile_address];
                    // write it in the framebuffer (if not 0)
                    if tile_pixel_color != 0
                        && sprite_x + x < self.framebuffer.width
                        && sprite_y + y < self.framebuffer.height
                    {
                        self.write_pixel(
                            (sprite_x + x) as u8,
                            (sprite_y + y) as u8,
                            tile_pixel_color,
                        );
                    }
                }
            }
        }
        self.framebuffer
            .blit(&self.screen, 0, 0, self.screen.width, self.screen.height);
        self.screen.swap();

        let mut input_state = self.input;
        let mut exit = false;

        self.screen.poll_events(|event| match event {
            WindowEvent::CloseRequested => exit = true,
            WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
                Some(VirtualKeyCode::Escape) => exit = true,
                Some(VirtualKeyCode::Up) => {
                    if input.state == ElementState::Pressed {
                        input_state |= 0x01;
                    } else {
                        input_state &= !0x01;
                    }
                }
                Some(VirtualKeyCode::Down) => {
                    if input.state == ElementState::Pressed {
                        input_state |= 0x02;
                    } else {
                        input_state &= !0x02;
                    }
                }
                Some(VirtualKeyCode::Right) => {
                    if input.state == ElementState::Pressed {
                        input_state |= 0x04;
                    } else {
                        input_state &= !0x04;
                    }
                }
                Some(VirtualKeyCode::Left) => {
                    if input.state == ElementState::Pressed {
                        input_state |= 0x08;
                    } else {
                        input_state &= !0x08;
                    }
                }
                Some(VirtualKeyCode::Space) => {
                    if input.state == ElementState::Pressed {
                        input_state |= 0x10;
                    } else {
                        input_state &= !0x10;
                    }
                }
                Some(VirtualKeyCode::LShift) => {
                    if input.state == ElementState::Pressed {
                        input_state |= 0x20;
                    } else {
                        input_state &= !0x20;
                    }
                }
                Some(VirtualKeyCode::RShift) => {
                    if input.state == ElementState::Pressed {
                        input_state |= 0x40;
                    } else {
                        input_state &= !0x40;
                    }
                }
                Some(VirtualKeyCode::LAlt) => {
                    if input.state == ElementState::Pressed {
                        input_state |= 0x80;
                    } else {
                        input_state &= !0x80;
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
        // first 4k are for the background
        match address {
            0x0000..=0x03ff => self.background0[address as usize] = value,
            0x0400..=0x07ff => self.background1[address as usize - 0x0400] = value,
            0x0800..=0x0bff => self.background2[address as usize - 0x0800] = value,
            0x0c00..=0x0fff => self.background3[address as usize - 0x0c00] = value,
            // a whole page is for sprite management (4 bytes for each sprite)
            // a total of 64 sprites is supported
            0x1000..=0x10ff => {
                let sprite_index = (address - 0x1000) / 4;
                let sprite_item = (address - 0x1000) % 4;
                let sprite = &mut self.sprites[sprite_index as usize];
                match sprite_item {
                    0 => sprite.tile = value,
                    1 => sprite.x = value,
                    2 => sprite.y = value,
                    3 => sprite.flags = value,
                    _ => (),
                }
            }
            0x1100 => self.background_color = value,
            0x1101 => self.scroll_x = value,
            0x1102 => self.scroll_y = value,
            0x1103 => self.current_col = value,
            0x1104 => self.current_row = value,
            0x1105 => {
                self.chr_ram[self.current_row as usize * 256 + self.current_col as usize] = value
            }
            0x1107 => self.background_mode = value,
            _ => (),
        }
    }

    fn read(&mut self, address: u16) -> u8 {
        // background
        match address {
            0x0000..=0x03ff => self.background0[address as usize],
            0x0400..=0x07ff => self.background1[address as usize - 0x0400],
            0x0800..=0x0bff => self.background2[address as usize - 0x0800],
            0x0c00..=0x0fff => self.background3[address as usize - 0x0c00],
            0x1000..=0x10ff => {
                let sprite_index = (address - 0x1000) / 4;
                let sprite_item = (address - 0x1000) % 4;
                let sprite = &mut self.sprites[sprite_index as usize];
                match sprite_item {
                    0 => sprite.tile,
                    1 => sprite.x,
                    2 => sprite.y,
                    3 => sprite.flags,
                    _ => 0,
                }
            }
            // gpu registers (0x1100) allow writing to chr ram
            0x1100 => self.background_color,
            0x1101 => self.scroll_x,
            0x1102 => self.scroll_y,
            0x1103 => self.current_col,
            0x1104 => self.current_row,
            0x1105 => self.chr_ram[self.current_row as usize * 256 + self.current_col as usize],
            0x1106 => self.input,
            0x1107 => self.background_mode,
            _ => 0,
        }
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
            Arg::with_name("code-breakpoint")
                .long("code-breakpoint")
                .help("enable code-driven breakpoints"),
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
        ).arg(
            Arg::with_name("chr-rom")
                .required(false)
                .long("chr-rom")
                .takes_value(true)
                .value_name("file")
                .help("attach a chr rom to the graphics ram"),
        ).arg(
            Arg::with_name("breakpoint")
                .required(false)
                .long("breakpoint")
                .takes_value(true)
                .value_name("address")
                .help("set breakpoint to the specified comma separated list of addresses"),
        ).arg(Arg::with_name("romfile").index(1).required(true))
        .get_matches();

    let romfile = matches.value_of("romfile").unwrap();

    let pc: u16 = match to_number(matches.value_of("pc").unwrap()) {
        Ok(value) => value,
        Err(_) => panic!("invalid address format for pc"),
    };

    let mut hz: u32 = match to_number(matches.value_of("hz").unwrap()) {
        Ok(value) => value,
        Err(_) => panic!("invalid number format for hz"),
    };

    if hz < 60 {
        println!("WARNING: forcing hz to minimal value: 60");
        hz = 60;
    }

    let piano_speed: u64 = match to_number(matches.value_of("piano-speed").unwrap()) {
        Ok(value) => value,
        Err(_) => panic!("invalid number format for piano-speed"),
    };

    let mut breakpoints: Vec<u16> = Vec::new();
    if matches.is_present("breakpoint") {
        let breakpoint_addresses = matches.value_of("breakpoint").unwrap().split(",");
        for breakpoint_address in breakpoint_addresses {
            breakpoints.push(to_number(breakpoint_address).unwrap());
        }
    }

    let mut rom = Rom::new(fs::read(romfile).unwrap());

    let mut ram = Ram::new(4096);

    let mut term8 = UnixTerm::new();

    let mut term = BusAdapter::new(&mut term8);

    let mut piano = Piano::new(piano_speed);

    let mut random = Random::new();

    let screen = Screen::new("aivmachine", 512, 512);

    let framebuffer = Framebuffer::new(256, 256);

    let aiv_framebuffer = Rc::new(RefCell::new(AivFrameBuffer::new(screen, framebuffer)));

    if matches.is_present("chr-rom") {
        let chrfile = matches.value_of("chr-rom").unwrap();
        let mut index = 0;
        let chr_ram = &mut aiv_framebuffer.borrow_mut().chr_ram;
        // here we use a cycle as the chr file could be < 65536
        for pixel in fs::read(chrfile).unwrap() {
            chr_ram[index] = pixel;
            index += 1;
        }
    }

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

    let mut in_debugger = false;

    cpu.set_code_breakpoint(matches.is_present("code-breakpoint"));

    loop {
        let mut ticks_counter = ticks_per_frame as i64;
        while ticks_counter > 0 {
            if cpu.is_code_breakpoint_requested() || breakpoints.contains(&cpu.pc) {
                in_debugger = true;
            }
            if in_debugger {
                in_debugger = debugger(&mut cpu);
            }
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
