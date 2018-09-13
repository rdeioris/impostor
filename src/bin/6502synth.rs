use std::cell::RefCell;
use std::env;
use std::fs;
use std::rc::Rc;

extern crate impostor;

use impostor::memcontroller::MemoryControllerShared;
use impostor::mos6502::MOS6502;
use impostor::ram::Ram;
use impostor::rom::Rom;
use impostor::synth::ChipTune;
use impostor::timer::SimpleTimer;

use impostor::Clock;

fn main() {
    let args: Vec<String> = env::args().collect();

    let rom = Rom::new(fs::read(&*args[1]).unwrap());

    let ram = Ram::new(32768);

    let wave_ram = Rc::new(RefCell::new(Ram::new(16384)));

    let mut cpu_memory_controller = MemoryControllerShared::new();
    let cpu_wave_ram = Rc::clone(&wave_ram);
    cpu_memory_controller.map(0xb000, 0xcfff, cpu_wave_ram);

    let mut audio_memory_controller = MemoryControllerShared::new();
    let audio_wave_ram = Rc::clone(&wave_ram);
    audio_memory_controller.map(0x0000, 0x1fff, audio_wave_ram);

    wave_ram.borrow_mut().fill(fs::read(&*args[2]).unwrap(), 0);

    let chip_tune = ChipTune::new(Box::new(audio_memory_controller));

    cpu_memory_controller.map(0x0000, 0x7fff, Rc::new(RefCell::new(ram)));
    cpu_memory_controller.map(0x8000, 0x8fff, Rc::new(RefCell::new(rom)));
    cpu_memory_controller.map(0x9000, 0xafff, Rc::new(RefCell::new(chip_tune)));

    let timer = SimpleTimer::new();

    cpu_memory_controller.map(0xd000, 0xd000, Rc::new(RefCell::new(timer)));

    let mut cpu = MOS6502::new(cpu_memory_controller);
    cpu.pc = 0x8000;
    cpu.debug = true;

    loop {
        cpu.step();
        //println!("[{:04X}] {}", cpu.debug_pc, cpu.debug_line);
    }
}
