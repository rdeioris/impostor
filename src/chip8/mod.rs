use rand;
use {AddressBusIO, Clock};

pub struct Chip8<T: AddressBusIO<u16, u8>> {
    bus: T,

    pub reg: [u8; 16],
    pub index: u16,

    pub screen: [u8; 64 * 32],

    pub keys: [bool; 16],

    pub pc: u16,

    pub delay_timer: u8,
    pub sound_timer: u8,

    pub stack: [u16; 16],
    pub sp: u8,

    pub redraw: bool,
}

impl<T: AddressBusIO<u16, u8>> Chip8<T> {
    pub fn new(bus: T) -> Chip8<T> {
        Chip8 {
            reg: [0; 16],
            pc: 0x200,
            stack: [0; 16],
            sp: 0xf,
            index: 0,
            delay_timer: 0,
            sound_timer: 0,
            screen: [0; 64 * 32],
            keys: [false; 16],
            redraw: false,
            bus: bus,
        }
    }

    fn read8(&mut self, addr: u16) -> u8 {
        self.bus.read(addr)
    }

    fn write8(&mut self, addr: u16, value: u8) {
        self.bus.write(addr, value)
    }

    fn read8_from_pc(&mut self) -> u8 {
        let pc = self.advance_pc();
        return self.read8(pc);
    }

    fn read16_from_pc(&mut self) -> u16 {
        let high = self.read8_from_pc() as u16;
        let low = self.read8_from_pc() as u16;
        return (high << 8) | low;
    }

    fn advance_pc(&mut self) -> u16 {
        let pc = self.pc;
        self.pc += 1;
        return pc;
    }
}

impl<T: AddressBusIO<u16, u8>> Clock for Chip8<T> {
    fn step(&mut self) {
        let opcode = self.read16_from_pc();

        let nnn = opcode & 0x0fff;
        let nn = (opcode & 0x00ff) as u8;
        let n = (opcode & 0x000f) as u8;
        let x = ((opcode & 0x0f00) >> 8) as usize;
        let y = ((opcode & 0x00f0) >> 4) as usize;

        match opcode & 0xf000 {
            0x0000 => match opcode & 0x00ff {
                0x00e0 => self.screen = [0; 64 * 32],
                0x00ee => {
                    if self.sp == 0x0f {
                        panic!("stack out of bounds");
                    }
                    self.sp += 1;
                    self.pc = self.stack[self.sp as usize];
                }
                _ => panic!("invalid opcode ${:04X}", opcode),
            },
            0x1000 => self.pc = nnn,
            0x2000 => {
                let sp = self.sp;
                if sp == 0 {
                    panic!("stack out of bounds");
                }
                self.stack[sp as usize] = self.pc;
                self.sp -= 1;
                self.pc = nnn;
            }
            0x3000 => {
                if self.reg[x] == nn {
                    self.pc += 2
                }
            }
            0x4000 => {
                if self.reg[x] != nn {
                    self.pc += 2
                }
            }
            0x5000 => {
                if self.reg[x] == self.reg[y] {
                    self.pc += 2
                }
            }
            0x6000 => self.reg[x] = nn,
            0x7000 => self.reg[x] += nn,
            0x8000 => match opcode & 0x000f {
                0x0000 => self.reg[x] = self.reg[y],
                0x0001 => self.reg[x] |= self.reg[y],
                0x0002 => self.reg[x] &= self.reg[y],
                0x0003 => self.reg[x] ^= self.reg[y],
                0x0004 => {
                    let a = self.reg[x] as u16;
                    let b = self.reg[y] as u16;
                    self.reg[0xf] = 0;
                    if a + b > 255 {
                        self.reg[0xf] = 1;
                    }
                    self.reg[x] += self.reg[y];
                }
                0x0005 => {
                    self.reg[0xf] = 1;
                    if self.reg[y] > self.reg[x] {
                        self.reg[0xf] = 0;
                    }
                    self.reg[x] -= self.reg[y];
                }
                0x0006 => {
                    self.reg[0xf] = self.reg[x] & 0x01;
                    self.reg[x] >>= 1;
                }
                _ => panic!("invalid opcode ${:04X}", opcode),
            },
            0x9000 => {
                if self.reg[x] != self.reg[y] {
                    self.pc += 2
                }
            }
            0xa000 => self.index = nnn,
            0xb000 => self.pc = nnn + (self.reg[0] as u16),
            0xc000 => self.reg[x] = rand::random::<u8>() & nn,
            0xd000 => {
                self.redraw = true;
                // first clear collision reg
                self.reg[0xf] = 0x00;
                for i in 0..n {
                    let index = self.index + i as u16;
                    let pixels = self.read8(index);
                    let pixel_y = self.reg[y] + i;
                    if pixel_y >= 32 {
                        break;
                    }
                    for j in 0..8 {
                        let pixel_x = self.reg[x] + j;
                        if pixel_x >= 64 {
                            break;
                        }
                        let offset = pixel_y as usize * 64 + pixel_x as usize;

                        if pixels & (0x80 >> j) != 0 {
                            if self.screen[offset] == 1 {
                                self.reg[0xf] = 0x01;
                            }
                            self.screen[offset] ^= 1;
                        }
                    }
                }
            }
            0xe000 => match opcode & 0x00ff {
                0x9e => {
                    if self.keys[self.reg[x] as usize] {
                        self.pc += 2
                    }
                }
                0xa1 => {
                    if !self.keys[self.reg[x] as usize] {
                        self.pc += 2
                    }
                }
                _ => panic!("invalid opcode ${:04X}", opcode),
            },
            0xf000 => match opcode & 0x00ff {
                0x0007 => self.reg[x] = self.delay_timer,
                0x0015 => self.delay_timer = self.reg[x],
                0x0018 => self.sound_timer = self.reg[x],
                0x000a => {
                    self.pc -= 2;
                    for i in 0..16 {
                        if self.keys[i] {
                            self.reg[x] = i as u8;
                            self.pc += 2;
                            break;
                        }
                    }
                }
                0x0033 => {
                    let index = self.index;
                    let value = self.reg[x];
                    self.write8(index, value / 100);
                    self.write8(index + 1, (value / 10) % 10);
                    self.write8(index + 2, (value % 100) % 10);
                }
                0x0065 => {
                    let mut index = self.index;
                    for i in 0..=x {
                        self.reg[i] = self.read8(index);
                        index += 1;
                    }
                }
                0x001e => self.index += self.reg[x] as u16,
                0x0029 => {
                    let offset = self.reg[x] as u16;
                    self.index = offset * 5;
                }
                _ => panic!("invalid opcode ${:04X}", opcode),
            },
            _ => panic!("invalid opcode ${:04X}", opcode),
        }
    }
}
