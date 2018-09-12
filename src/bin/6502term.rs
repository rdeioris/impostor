use std::env;
use std::fs;

extern crate impostor;

use impostor::memcontroller::MemoryController;
use impostor::mos6502::MOS6502;
use impostor::ram::Ram;
use impostor::rom::Rom;
use impostor::unixterm::UnixTerm;

use impostor::Clock;

fn main() {
    let args: Vec<String> = env::args().collect();

    let rom = Rom::new(fs::read(&*args[1]).unwrap());

    let ram = Ram::new(4096);

    let term = UnixTerm::new();

    let mut memory_controller = MemoryController::new();
    memory_controller.map(0x0000, 0x0fff, Box::new(ram));
    memory_controller.map(0x8000, 0x8fff, Box::new(rom));
    memory_controller.map(0x2000, 0x2007, Box::new(term));

    let mut cpu = MOS6502::new(memory_controller);
    cpu.pc = 0x8000;
    cpu.debug = true;

    loop {
        cpu.step();
        //println!("[{:04X}] {}", cpu.debug_pc, cpu.debug_line);
    }
}
