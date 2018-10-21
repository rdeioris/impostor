extern crate clap;
extern crate impostor;

use clap::{App, Arg};

use impostor::adapter::BusAdapter;
use impostor::audio::Beeper;
use impostor::memcontroller::MemoryControllerSmart;
use impostor::mos6502::MOS6502;
use impostor::ram::Ram;
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

struct AivFrameBuffer {
    framebuffer: Framebuffer,
    screen: Screen,
    current_row: u8,
    current_col: u8,
    scroll_x: u8,
    scroll_y: u8,
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
        }
    }

    fn vblank(&mut self) -> bool {
        let mut exit = false;
        let x = self.scroll_x as usize * (self.screen.width / self.framebuffer.width);
        let y = self.scroll_y as usize * (self.screen.height / self.framebuffer.height);
        self.screen.clear();
        self.framebuffer
            .blit(x, y, self.screen.width, self.screen.height);
        self.screen.swap();
        self.screen.poll_events(|event| match event {
            WindowEvent::CloseRequested => exit = true,
            _ => (),
        });
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
        match address {
            256 => value = self.current_row,
            258 => value = self.scroll_x,
            259 => value = self.scroll_y,
            260 => value = self.current_col,
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

    let mut rom = Rom::new(fs::read(romfile).unwrap());

    let mut ram = Ram::new(4096);

    let mut term8 = UnixTerm::new();

    let mut term = BusAdapter::new(&mut term8);

    let mut beeper = Beeper::new(880);

    let screen = Screen::new("aivmachine", 512, 512);

    let framebuffer = Framebuffer::new(256, 256);

    let aiv_framebuffer = Rc::new(RefCell::new(AivFrameBuffer::new(screen, framebuffer)));

    let mut memory_controller = MemoryControllerSmart::new();
    memory_controller.map(0x0000, 0x1fff, &mut ram);
    memory_controller.map(0x2000, 0x2003, &mut term);
    memory_controller.map(0x2004, 0x2004, &mut beeper);

    let borrowed_aiv_framebuffer = Rc::clone(&aiv_framebuffer);
    memory_controller.map_shared(0x4000, 0x41ff, borrowed_aiv_framebuffer);

    memory_controller.map(0xc000, 0xffff, &mut rom);

    let mut cpu = MOS6502::new(memory_controller);
    cpu.pc = pc;
    cpu.debug = matches.is_present("debug");

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
        cpu.raise(6);
    }
}