extern crate impostor;

use impostor::mos6502::MOS6502;
use impostor::ram::Ram;
use impostor::unixterm::UnixTerm;

use impostor::Clock;

fn main() {
    let term = UnixTerm::new();
    let ram = Ram::new(4096);
    let mut cpu = MOS6502::new(ram);
    cpu.step();
}
