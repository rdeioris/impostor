use std::env;
use std::fs;

extern crate impostor;

use impostor::memcontroller::MemoryController;
use impostor::mos6502::MOS6502;
use impostor::ram::Ram;
use impostor::rom::Rom;
use impostor::AddressBusIO;

use impostor::Clock;

struct TIA {
}

impl TIA {
    fn new() -> TIA {
        TIA{}
    }
}

impl AddressBusIO<u16, u8> for TIA {
    fn write(&mut self, address: u16, data: u8) {
        println!("writing #${:02X} to ${:04X}", data, address);
    }
}

struct RIOT {
}

impl RIOT {
    fn new() -> RIOT {
        RIOT{}
    }
}

impl AddressBusIO<u16, u8> for RIOT {
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut rom = Rom::new(fs::read(&*args[1]).unwrap());

    let mut ram = Ram::new(128);

    let mut tia = TIA::new();

    let mut riot = RIOT::new();

    let mut memory_controller = MemoryController::new();
    memory_controller.panic_on_no_map = true;
    memory_controller.map(0x0000, 0x003f, &mut tia);
    memory_controller.map(0x0080, 0x00ff, &mut ram);
    memory_controller.map(0x0280, 0x029f, &mut riot);
    memory_controller.map(0x1000, 0x1fff, &mut rom);

    memory_controller.mirror(0x0040, 0x007f, 0x1000);

    memory_controller.mirror(0x0180, 0x01ff, 0x0080);

    memory_controller.mirror(0x3000, 0x3fff, 0x1000);
    memory_controller.mirror(0x5000, 0x5fff, 0x1000);
    memory_controller.mirror(0x7000, 0x7fff, 0x1000);
    memory_controller.mirror(0x9000, 0x9fff, 0x1000);
    memory_controller.mirror(0xb000, 0xbfff, 0x1000);
    memory_controller.mirror(0xd000, 0xdfff, 0x1000);
    memory_controller.mirror(0xf000, 0xffff, 0x1000);

    let mut cpu = MOS6502::new(memory_controller);
    cpu.pc = 0xf000;
    cpu.debug = true;

    loop {
        cpu.step();
        println!("[{:04X}] {}", cpu.debug_pc, cpu.debug_line);
    }
}
