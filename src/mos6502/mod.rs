use {AddressBusIO, Clock};

const CARRY: u8 = 0x01;
const ZERO: u8 = 0x02;
const INTERRUPT: u8 = 0x04;
const DECIMAL: u8 = 0x08;
const BRK: u8 = 0x10;
const OVERFLOW: u8 = 0x40;
const SIGN: u8 = 0x80;

struct OpCode<T: AddressBusIO<u16, u8>> {
    fetch: fn(&mut MOS6502<T>),
    fun: fn(&mut MOS6502<T>),
    name: &'static str,
}

// we cannot use derive as the generics in place generates mess
impl<T: AddressBusIO<u16, u8>> Copy for OpCode<T> { }

impl<T: AddressBusIO<u16, u8>> Clone for OpCode<T> {
    fn clone(&self) -> OpCode<T> {
        *self
    }
}

pub struct MOS6502<T: AddressBusIO<u16, u8>> {
    bus: T,

    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub pc: u16,
    pub sp: u8,
    pub status: u8,

    pub debug: bool,
    pub debug_line: String,

    pub ticks: u64,

    value: u8,
    addr: u16,

    current_opcode: u8,
    opcode: OpCode<T>,

    opcodes: [OpCode<T>; 256],
}

impl<T: AddressBusIO<u16, u8>> MOS6502<T> {
    pub fn new(bus: T) -> MOS6502<T> {
        let noop = OpCode {
            fetch: MOS6502::invalid,
            fun: MOS6502::nop,
            name: "-",
        };

        let mut cpu = MOS6502 {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            sp: 0xff,
            value: 0,
            addr: 0,
            ticks: 0,
            opcode: noop,
            current_opcode: 0,
            debug_line: "".to_string(),

            opcodes: [noop; 256],

            debug: false,

            status: 0x20,

            bus: bus,
        };

        cpu.register_opcode(
            "ADC",
            Self::adc,
            &[
                (0x69, Self::immediate),
                (0x65, Self::zeropage),
                (0x75, Self::zeropage_x),
                (0x6d, Self::absolute),
                (0x7d, Self::absolute_x),
                (0x79, Self::absolute_y),
                (0x61, Self::indirect_x),
                (0x71, Self::indirect_y),
            ],
        );

        return cpu;
    }

    fn register_opcode(
        &mut self,
        name: &'static str,
        fun: fn(&mut MOS6502<T>),
        opcodes: &[(u8, fn(&mut MOS6502<T>))],
    ) {
        for opcode in opcodes {
            self.opcodes[opcode.0 as usize] = OpCode {
                fetch: opcode.1,
                fun: fun,
                name: name,
            };
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
        let low = self.read8_from_pc() as u16;
        let high = self.read8_from_pc() as u16;
        return (high << 8) | low;
    }

    fn advance_pc(&mut self) -> u16 {
        let pc = self.pc;
        self.pc += 1;
        return pc;
    }

    fn get_opcode_name(&self) -> &'static str {
        self.opcode.name
    }

    fn implied(&mut self) {
        self.ticks += 2;
        if self.debug {
            self.debug_line = self.get_opcode_name().to_string()
        }
    }

    fn immediate(&mut self) {
        self.value = self.read8_from_pc();
        self.ticks += 2;
        if self.debug {
            self.debug_line = format!("{} #${:02X}", self.get_opcode_name(), self.value);
        }
    }

    fn relative(&mut self) {
        let offset = self.read8_from_pc() as i8;
        self.ticks += 2;
        let addr: i32 = self.pc as i32 + offset as i32;
        self.addr = addr as u16;
        if self.debug {
            self.debug_line = format!("{} ${:04X}", self.get_opcode_name(), self.addr);
        }
    }

    fn zeropage(&mut self) {
        let addr = self.read8_from_pc() as u16;
        self.addr = addr;
        self.value = self.read8(addr);
        self.ticks += 3;
        self.debug_line = format!("{} ${:02X}", self.get_opcode_name(), self.addr);
    }

    fn absolute(&mut self) {
        let addr = self.read16_from_pc();
        self.addr = addr;
        self.value = self.read8(addr);
        self.ticks += 4;
        if self.debug {
            self.debug_line = format!("{} ${:04X}", self.get_opcode_name(), self.addr);
        }
    }

    fn absolute_x(&mut self) {
        let addr = self.read16_from_pc();
        let mut boundary = 0;
        let addr_x = addr + self.x as u16;
        if addr >> 8 != addr_x >> 8 {
            boundary = 1;
        }
        self.addr = addr_x;
        self.value = self.read8(addr_x);
        self.ticks += 4 + boundary;
        if self.debug {
            self.debug_line = format!("{} ${:04X},X", self.get_opcode_name(), self.addr);
        }
    }

    fn absolute_y(&mut self) {
        let addr = self.read16_from_pc();
        let mut boundary = 0;
        let addr_y = addr + self.y as u16;
        if addr >> 8 != addr_y >> 8 {
            boundary = 1;
        }
        self.addr = addr_y;
        self.value = self.read8(addr_y);
        self.ticks += 4 + boundary;
        if self.debug {
            self.debug_line = format!("{} ${:04X},Y", self.get_opcode_name(), self.addr);
        }
    }

    fn zeropage_x(&mut self) {
        let pc = self.pc;
        // leave it as u8 to allow overflowing
        let offset = self.read8(pc) + self.x;
        self.addr = offset as u16;
        self.value = self.read8(offset as u16);
        self.pc += 1;
        self.ticks += 3;
        if self.debug {
            self.debug_line = format!("{} ${:02X},X", self.get_opcode_name(), self.addr);
        }
    }

    fn indirect_x(&mut self) {
        let pc = self.pc;
        // leave it as u8 to allow overflowing
        let offset = (self.read8(pc) + self.x) as u16;
        self.addr = offset;
        let indirect_addr = self.read8(offset) as u16;
        self.value = self.read8(indirect_addr);
        self.pc += 1;
        self.ticks += 2;
        if self.debug {
            self.debug_line = format!("{} (${:02X},X)", self.get_opcode_name(), self.addr);
        }
    }

    fn indirect_y(&mut self) {
        let pc = self.pc;
        // leave it as u8 to allow overflowing
        let offset = self.read8(pc) as u16;
        self.addr = offset;
        let indirect_addr = (self.read8(offset) as u16) + self.y as u16;
        self.value = self.read8(indirect_addr);
        self.pc += 1;
        self.ticks += 2;
        if indirect_addr >> 8 != 0 {
            self.ticks += 1;
        }
        if self.debug {
            self.debug_line = format!("{} (${:02X}),Y", self.get_opcode_name(), self.addr);
        }
    }

    fn get_flag(&self, flag: u8) -> bool {
        return (self.status & flag) != 0;
    }

    fn set_flag(&mut self, flag: u8, enabled: bool) {
        if enabled {
            self.status |= flag;
        } else {
            self.status &= !flag;
        }
    }

    // OPCODES

    fn lda(&mut self) {
        let a = self.value;
        self.a = a;
        self.set_flag(ZERO, a == 0);
        self.set_flag(SIGN, a >> 7 == 1);
    }

    fn tay(&mut self) {
        self.y = self.a;
        let y = self.y;
        self.set_flag(ZERO, y == 0);
        self.set_flag(SIGN, y >> 7 == 1);
    }

    fn tax(&mut self) {
        self.x = self.a;
        let x = self.x;
        self.set_flag(ZERO, x == 0);
        self.set_flag(SIGN, x >> 7 == 1);
    }

    fn txa(&mut self) {
        self.a = self.x;
        let a = self.a;
        self.set_flag(ZERO, a == 0);
        self.set_flag(SIGN, a >> 7 == 1);
    }

    fn tya(&mut self) {
        self.a = self.y;
        let a = self.a;
        self.set_flag(ZERO, a == 0);
        self.set_flag(SIGN, a >> 7 == 1);
    }

    fn iny(&mut self) {
        self.y += 1;
        let y = self.y;
        self.set_flag(ZERO, y == 0);
        self.set_flag(SIGN, y >> 7 == 1);
    }

    fn inx(&mut self) {
        self.x += 1;
        let x = self.x;
        self.set_flag(ZERO, x == 0);
        self.set_flag(SIGN, x >> 7 == 1);
    }

    fn ldy(&mut self) {
        let y = self.value;
        self.y = y;
        self.set_flag(ZERO, y == 0);
        self.set_flag(SIGN, y >> 7 == 1);
    }

    fn stx(&mut self) {
        let addr = self.addr;
        let x = self.x;
        self.write8(addr, x);
    }

    fn sty(&mut self) {
        let addr = self.addr;
        let y = self.y;
        self.write8(addr, y);
    }

    fn sta(&mut self) {
        let addr = self.addr;
        let a = self.a;
        self.write8(addr, a);
    }

    fn ldx(&mut self) {
        self.x = self.value;
        let x = self.x;
        self.set_flag(ZERO, x == 0);
        self.set_flag(SIGN, x >> 7 == 1);
    }

    fn and(&mut self) {
        self.a &= self.value;
        let a = self.a;
        self.set_flag(ZERO, a == 0);
        self.set_flag(SIGN, a >> 7 == 1);
    }

    fn sbc(&mut self) {
        // first check for carry
        let carry = if self.get_flag(CARRY) { 0 } else { 1 };
        let orig_a: i16 = self.a as i16;
        let value: i16 = self.value as i16;
        let result: i16 = orig_a - value - carry;
        self.set_flag(CARRY, result >= 0 && result <= 0xff);
        self.a = result as u8;
        let a = self.a;
        self.set_flag(ZERO, a == 0);
        self.set_flag(SIGN, a >> 7 == 1);
        // if the sign of both inputs is different from the sign of the result
        self.set_flag(
            OVERFLOW,
            ((orig_a as u8 ^ a) & (value as u8 ^ a)) & 0x80 != 0,
        );
    }

    fn adc(&mut self) {
        // first check for carry
        let carry = if self.get_flag(CARRY) { 1 } else { 0 };
        let orig_a: i16 = self.a as i16;
        let value: i16 = self.value as i16;
        let result: i16 = orig_a + value + carry;
        self.set_flag(CARRY, result > 0xff);
        self.a = result as u8;
        let a = self.a;
        self.set_flag(ZERO, a == 0);
        self.set_flag(SIGN, a >> 7 == 1);
        // if the sign of both inputs is different from the sign of the result
        self.set_flag(
            OVERFLOW,
            ((orig_a as u8 ^ a) & (value as u8 ^ a)) & 0x80 != 0,
        );
    }

    fn jmp(&mut self) {
        self.pc = self.addr;
    }

    fn sec(&mut self) {
        self.set_flag(CARRY, true);
    }

    fn clc(&mut self) {
        self.set_flag(CARRY, false);
    }

    fn beq(&mut self) {
        if self.get_flag(ZERO) {
            self.pc = self.addr;
            self.ticks += 1;
        }
    }

    fn bcs(&mut self) {
        if self.get_flag(CARRY) {
            self.pc = self.addr;
            self.ticks += 1;
        }
    }

    fn bcc(&mut self) {
        if !self.get_flag(CARRY) {
            self.pc = self.addr;
            self.ticks += 1;
        }
    }

    fn pha(&mut self) {
        let sp: u16 = 0x100 + (self.sp as u16);
        let a = self.a;
        self.write8(sp, a);
        self.sp -= 1;
        self.ticks += 1;
    }

    fn pla(&mut self) {
        self.sp += 1;
        let sp: u16 = 0x100 + (self.sp as u16);
        self.a = self.read8(sp);
        self.ticks += 2;
    }

    fn php(&mut self) {
        let sp: u16 = 0x100 + (self.sp as u16);
        let status = self.status;
        self.write8(sp, status);
        self.sp -= 1;
        self.ticks += 1;
    }

    fn plp(&mut self) {
        self.sp += 1;
        let sp: u16 = 0x100 + (self.sp as u16);
        self.status = self.read8(sp);
        self.ticks += 2;
    }

    fn jsr(&mut self) {
        let sp: u16 = 0x100 + (self.sp as u16);
        let pc = self.pc - 1;
        let pc_high = (pc >> 8) as u8;
        let pc_low = (pc & 0x00ff) as u8;
        self.write8(sp, pc_high);
        self.write8(sp - 1, pc_low);
        self.sp -= 2;

        self.pc = self.addr;
        self.ticks += 2;
    }

    fn rts(&mut self) {
        self.sp += 1;
        let sp: u16 = 0x100 + (self.sp as u16);
        let pc_low: u16 = self.read8(sp) as u16;
        let pc_high: u16 = self.read8(sp + 1) as u16;
        self.sp += 1;
        self.pc = (pc_high << 8 | pc_low) + 1;
        //println!("RTS: {:04X}", self.pc);
        self.ticks += 4;
    }

    fn nop(&mut self) {}

    fn invalid(&mut self) {
        panic!("invalid opcode {:02X}", self.current_opcode);
    }
}

impl<T: AddressBusIO<u16, u8>> Clock for MOS6502<T> {
    fn step(&mut self) {
        let pc = self.pc;
        let opcode = self.read8_from_pc();
        self.current_opcode = opcode;
        self.opcode = self.opcodes[opcode as usize];
        // fetch
        (self.opcode.fetch)(self);
        // execute
        (self.opcode.fun)(self);
    }
}
