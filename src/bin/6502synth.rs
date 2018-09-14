use std::env;
use std::fs;
use std::sync::{Arc, Mutex};

extern crate impostor;

use impostor::memcontroller::MemoryControllerThreadSafe;
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

    let wave_ram = Arc::new(Mutex::new(Ram::new(16384)));

    let mut cpu_memory_controller = MemoryControllerThreadSafe::new();
    cpu_memory_controller.map(0xb000, 0xcfff, wave_ram.clone());

    let mut audio_memory_controller = MemoryControllerThreadSafe::new();
    audio_memory_controller.map(0x0000, 0x1fff, wave_ram.clone());

    wave_ram.lock().unwrap().fill(fs::read(&*args[2]).unwrap(), 0);

    let chip_tune = ChipTune::new(Arc::new(Mutex::new(audio_memory_controller)));

    cpu_memory_controller.map(0x0000, 0x7fff, Arc::new(Mutex::new(ram)));
    cpu_memory_controller.map(0x8000, 0x8fff, Arc::new(Mutex::new(rom)));
    cpu_memory_controller.map(0x9000, 0xafff, Arc::new(Mutex::new(chip_tune)));

    let timer = Arc::new(Mutex::new(SimpleTimer::new()));
    cpu_memory_controller.map(0xd000, 0xd000, timer.clone());

    let cpu = Arc::new(Mutex::new(MOS6502::new(cpu_memory_controller)));

    timer.lock().unwrap().connect_to_interrupt_line(cpu.clone(), 0x04);


    cpu.lock().unwrap().pc = 0x8000;
    cpu.lock().unwrap().debug = true;

    loop {
        cpu.lock().unwrap().step();
        //println!("[{:04X}] {}", cpu.debug_pc, cpu.debug_line);
    }
}
