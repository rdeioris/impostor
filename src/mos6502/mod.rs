use {AddressBusIO, Clock, Debug, Interrupt};

const CARRY: u8 = 0x01;
const ZERO: u8 = 0x02;
const INTERRUPT: u8 = 0x04;
const DECIMAL: u8 = 0x08;
const BRK: u8 = 0x10;
const ALWAYS_SET: u8 = 0x20;
const OVERFLOW: u8 = 0x40;
const SIGN: u8 = 0x80;

struct OpCode<T: AddressBusIO<u16, u8>> {
    fetch: fn(&mut MOS6502<T>),
    fun: fn(&mut MOS6502<T>),
    name: &'static str,
}

// we cannot use derive as the generics in place generates mess
impl<T: AddressBusIO<u16, u8>> Copy for OpCode<T> {}

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
    pub debug_pc: u16,

    pub ticks: u64,

    value: u8,
    addr: u16,

    code_breakpoint: bool,
    requested_code_breakpoint: bool,

    current_opcode: u8,
    opcode: OpCode<T>,

    opcodes: [OpCode<T>; 256],
}

macro_rules! opcode {
    ($cpu:ident, $name:ident, $code:expr, $fetch:ident) => (
        $cpu.register_opcode(stringify!($name), Self::$name, $code, Self::$fetch);
    );
    ($cpu:ident, $name:ident, $code:expr, $fetch:ident, $($codeN:expr, $fetchN:ident),+) => (
        opcode!($cpu, $name, $code, $fetch);
        opcode!($cpu, $name, $($codeN, $fetchN),+);
    );
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

            debug_pc: 0,
            debug_line: "".to_string(),

            opcodes: [noop; 256],

            code_breakpoint: false,
            requested_code_breakpoint: false,

            debug: false,

            status: ALWAYS_SET | INTERRUPT,

            bus: bus,
        };

        opcode!(
            cpu, adc, 0x69, immediate, 0x65, zeropage, 0x75, zeropage_x, 0x6d, absolute, 0x7d,
            absolute_x, 0x79, absolute_y, 0x61, indirect_x, 0x71, indirect_y
        );

        opcode!(cpu, bit, 0x24, zeropage, 0x2c, absolute);

        opcode!(
            cpu, and, 0x29, immediate, 0x25, zeropage, 0x35, zeropage_x, 0x2d, absolute, 0x3d,
            absolute_x, 0x39, absolute_y, 0x21, indirect_x, 0x31, indirect_y
        );

        opcode!(cpu, asl_a, 0x0a, accumulator);

        opcode!(cpu, lsr_a, 0x4a, accumulator);

        opcode!(cpu, asl, 0x06, zeropage, 0x16, zeropage_x, 0x0e, absolute, 0x1e, absolute_x);

        opcode!(
            cpu, eor, 0x49, immediate, 0x45, zeropage, 0x55, zeropage_x, 0x4d, absolute, 0x5d,
            absolute_x, 0x59, absolute_y, 0x41, indirect_x, 0x51, indirect_y
        );

        opcode!(cpu, lsr, 0x46, zeropage, 0x56, zeropage_x, 0x4e, absolute, 0x5e, absolute_x);

        opcode!(
            cpu, ora, 0x09, immediate, 0x05, zeropage, 0x15, zeropage_x, 0x0d, absolute, 0x1d,
            absolute_x, 0x19, absolute_y, 0x01, indirect_x, 0x11, indirect_y
        );

        opcode!(cpu, bpl, 0x10, relative);
        opcode!(cpu, bmi, 0x30, relative);

        opcode!(cpu, bvc, 0x50, relative);
        opcode!(cpu, bvs, 0x70, relative);

        opcode!(cpu, beq, 0xf0, relative);
        opcode!(cpu, bne, 0xd0, relative);

        opcode!(cpu, bcc, 0x90, relative);
        opcode!(cpu, bcs, 0xb0, relative);

        opcode!(cpu, brk, 0x00, implied);

        opcode!(
            cpu, cmp, 0xc9, immediate, 0xc5, zeropage, 0xd5, zeropage_x, 0xcd, absolute, 0xdd,
            absolute_x, 0xd9, absolute_y, 0xc1, indirect_x, 0xd1, indirect_y
        );
        opcode!(cpu, cpx, 0xe0, immediate, 0xe4, zeropage, 0xec, absolute);
        opcode!(cpu, cpy, 0xc0, immediate, 0xc4, zeropage, 0xcc, absolute);

        opcode!(cpu, dec, 0xc6, zeropage, 0xd6, zeropage_x, 0xce, absolute, 0xde, absolute_x);
        opcode!(cpu, inc, 0xe6, zeropage, 0xf6, zeropage_x, 0xee, absolute, 0xfe, absolute_x);

        opcode!(cpu, clc, 0x18, implied);
        opcode!(cpu, sec, 0x38, implied);
        opcode!(cpu, cli, 0x58, implied);
        opcode!(cpu, sei, 0x78, implied);
        opcode!(cpu, clv, 0xb8, implied);
        opcode!(cpu, cld, 0xd8, implied);
        opcode!(cpu, sed, 0xf8, implied);

        opcode!(cpu, jmp, 0x4c, absolute, 0x6c, indirect);
        opcode!(cpu, jsr, 0x20, absolute);

        opcode!(
            cpu, lda, 0xa9, immediate, 0xa5, zeropage, 0xb5, zeropage_x, 0xad, absolute, 0xbd,
            absolute_x, 0xb9, absolute_y, 0xa1, indirect_x, 0xb1, indirect_y
        );
        opcode!(
            cpu, ldx, 0xa2, immediate, 0xa6, zeropage, 0xb6, zeropage_y, 0xae, absolute, 0xbe,
            absolute_y
        );
        opcode!(
            cpu, ldy, 0xa0, immediate, 0xa4, zeropage, 0xb4, zeropage_x, 0xac, absolute, 0xbc,
            absolute_x
        );

        opcode!(cpu, nop, 0xea, implied);

        opcode!(cpu, tax, 0xaa, implied);
        opcode!(cpu, txa, 0x8a, implied);
        opcode!(cpu, dex, 0xca, implied);
        opcode!(cpu, inx, 0xe8, implied);
        opcode!(cpu, tay, 0xa8, implied);
        opcode!(cpu, tya, 0x98, implied);
        opcode!(cpu, tay, 0xa8, implied);
        opcode!(cpu, dey, 0x88, implied);
        opcode!(cpu, iny, 0xc8, implied);

        opcode!(cpu, rts, 0x60, implied);

        opcode!(cpu, rti, 0x40, implied);

        opcode!(
            cpu, sbc, 0xe9, immediate, 0xe5, zeropage, 0xf5, zeropage_x, 0xed, absolute, 0xfd,
            absolute_x, 0xf9, absolute_y, 0xe1, indirect_x, 0xf1, indirect_y
        );

        opcode!(
            cpu, sta, 0x85, zeropage, 0x95, zeropage_x, 0x8d, absolute, 0x9d, absolute_x, 0x99,
            absolute_y, 0x81, indirect_x, 0x91, indirect_y
        );

        opcode!(cpu, stx, 0x86, zeropage, 0x96, zeropage_x, 0x8e, absolute);
        opcode!(cpu, sty, 0x84, zeropage, 0x94, zeropage_x, 0x8c, absolute);

        opcode!(cpu, txs, 0x9a, implied);
        opcode!(cpu, tsx, 0xba, implied);
        opcode!(cpu, pha, 0x48, implied);
        opcode!(cpu, pla, 0x68, implied);
        opcode!(cpu, php, 0x08, implied);
        opcode!(cpu, plp, 0x28, implied);

        opcode!(cpu, rol_a, 0x2a, accumulator);

        opcode!(cpu, rol, 0x26, zeropage, 0x36, zeropage_x, 0x2e, absolute, 0x3e, absolute_x);

        opcode!(cpu, ror_a, 0x6a, accumulator);

        opcode!(cpu, ror, 0x66, zeropage, 0x76, zeropage_x, 0x6e, absolute, 0x7e, absolute_x);

        cpu
    }

    fn register_opcode(
        &mut self,
        name: &'static str,
        fun: fn(&mut MOS6502<T>),
        code: u8,
        fetch: fn(&mut MOS6502<T>),
    ) {
        self.opcodes[code as usize] = OpCode {
            fetch: fetch,
            fun: fun,
            name: name,
        };
    }

    fn read8(&mut self, addr: u16) -> u8 {
        self.bus.read(addr)
    }

    fn read16(&mut self, addr: u16) -> u16 {
        let low = u16::from(self.read8(addr));
        let high = u16::from(self.read8(addr + 1));
        (high << 8) | low
    }

    fn write8(&mut self, addr: u16, value: u8) {
        self.bus.write(addr, value)
    }

    fn read8_from_pc(&mut self) -> u8 {
        let pc = self.advance_pc();
        self.read8(pc)
    }

    fn read16_from_pc(&mut self) -> u16 {
        let low = u16::from(self.read8_from_pc());
        let high = u16::from(self.read8_from_pc());
        (high << 8) | low
    }

    fn advance_pc(&mut self) -> u16 {
        let pc = self.pc;
        self.pc += 1;
        pc
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

    fn accumulator(&mut self) {
        self.ticks += 2;
        if self.debug {
            self.debug_line = self.get_opcode_name().to_string();
        }
    }

    fn relative(&mut self) {
        let offset = self.read8_from_pc() as i8;
        self.ticks += 2;
        let addr = i32::from(self.pc) + i32::from(offset);
        self.addr = addr as u16;
        if self.debug {
            self.debug_line = format!("{} ${:04X}", self.get_opcode_name(), self.addr);
        }
    }

    fn zeropage(&mut self) {
        let addr = u16::from(self.read8_from_pc());
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
        let original_addr = addr;
        let mut boundary = 0;
        let addr_x = addr + u16::from(self.x);
        if addr >> 8 != addr_x >> 8 {
            boundary = 1;
        }
        self.addr = addr_x;
        self.value = self.read8(addr_x);
        self.ticks += 4 + boundary;
        if self.debug {
            self.debug_line = format!(
                "{} ${:04X},X (absolute addr: ${:04X})",
                self.get_opcode_name(),
                original_addr,
                self.addr
            );
        }
    }

    fn absolute_y(&mut self) {
        let addr = self.read16_from_pc();
        let original_addr = addr;
        let mut boundary = 0;
        let addr_y = addr + u16::from(self.y);
        if addr >> 8 != addr_y >> 8 {
            boundary = 1;
        }
        self.addr = addr_y;
        self.value = self.read8(addr_y);
        self.ticks += 4 + boundary;
        if self.debug {
            self.debug_line = format!(
                "{} ${:04X},Y (absolute addr: ${:04X})",
                self.get_opcode_name(),
                original_addr,
                self.addr
            );
        }
    }

    fn zeropage_x(&mut self) {
        // leave it as u8 to allow overflowing
        let original_addr = self.read8_from_pc();
        let addr = original_addr + self.x;
        self.addr = u16::from(addr);
        self.value = self.read8(u16::from(addr));
        self.ticks += 3;
        if self.debug {
            self.debug_line = format!(
                "{} ${:02X},X (zeropage addr: ${:02X})",
                self.get_opcode_name(),
                original_addr,
                self.addr
            );
        }
    }

    fn zeropage_y(&mut self) {
        // leave it as u8 to allow overflowing
        let original_addr = self.read8_from_pc();
        let addr = original_addr + self.y;
        self.addr = u16::from(addr);
        self.value = self.read8(u16::from(addr));
        self.ticks += 3;
        if self.debug {
            self.debug_line = format!(
                "{} ${:02X},Y (zeropage addr: ${:02X})",
                self.get_opcode_name(),
                original_addr,
                self.addr
            );
        }
    }

    fn indirect(&mut self) {
        let addr = self.read16_from_pc();
        let indirect_addr = self.read16(addr) as u16;
        self.addr = indirect_addr;
        self.value = self.read8(indirect_addr);
        self.pc += 1;
        self.ticks += 2;
        if self.debug {
            self.debug_line = format!(
                "{} (${:04X}) (indirect addr: ${:04X})",
                self.get_opcode_name(),
                addr,
                self.addr
            );
        }
    }

    fn indirect_x(&mut self) {
        let pc = self.pc;
        // leave it as u8 to allow overflowing
        let original_offset = u16::from(self.read8(pc));
        let offset = original_offset + 2;
        let indirect_addr = self.read16(offset);
        self.addr = indirect_addr;
        self.value = self.read8(indirect_addr);
        self.pc += 1;
        self.ticks += 3;
        if self.debug {
            self.debug_line = format!(
                "{} (${:02X},X) (indirect addr: ${:04X})",
                self.get_opcode_name(),
                original_offset,
                self.addr
            );
        }
    }

    fn indirect_y(&mut self) {
        let pc = self.pc;
        // leave it as u8 to allow overflowing
        let offset = u16::from(self.read8(pc));
        let indirect_addr = self.read16(offset) + u16::from(self.y);
        self.value = self.read8(indirect_addr);
        self.addr = indirect_addr;
        self.pc += 1;
        self.ticks += 2;
        if indirect_addr >> 8 != 0 {
            self.ticks += 1;
        }
        if self.debug {
            self.debug_line = format!(
                "{} (${:02X}),Y (indirect addr: ${:04X})",
                self.get_opcode_name(),
                offset,
                self.addr
            );
        }
    }

    fn get_flag(&self, flag: u8) -> bool {
        (self.status & flag) != 0
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

    fn dex(&mut self) {
        self.x -= 1;
        let x = self.x;
        self.set_flag(ZERO, x == 0);
        self.set_flag(SIGN, x >> 7 == 1);
    }

    fn dey(&mut self) {
        self.y -= 1;
        let y = self.y;
        self.set_flag(ZERO, y == 0);
        self.set_flag(SIGN, y >> 7 == 1);
    }

    fn dec(&mut self) {
        let value = self.value - 1;
        let addr = self.addr;
        self.write8(addr, value);
        self.set_flag(ZERO, value == 0);
        self.set_flag(SIGN, value >> 7 == 1);
    }

    fn inc(&mut self) {
        let value = self.value + 1;
        let addr = self.addr;
        self.write8(addr, value);
        self.set_flag(ZERO, value == 0);
        self.set_flag(SIGN, value >> 7 == 1);
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

    fn bit(&mut self) {
        let a = self.a;
        let value = self.value;
        self.set_flag(ZERO, (a & value) == 0);
        self.set_flag(SIGN, (value & 0x80) != 0);
        self.set_flag(OVERFLOW, (value & 0x40) != 0);
    }

    fn asl_a(&mut self) {
        let mut a = self.a;
        self.set_flag(CARRY, (a >> 7) == 0x01);
        a <<= 1;
        self.a = a;
        self.set_flag(ZERO, a == 0);
        self.set_flag(SIGN, a >> 7 == 1);
    }

    fn rol_a(&mut self) {
        let mut a = self.a;
        let carry = self.get_flag(CARRY);
        self.set_flag(CARRY, (a >> 7) == 0x01);
        a <<= 1;
        a |= if carry { 1 } else { 0 };
        self.a = a;
        self.set_flag(ZERO, a == 0);
        self.set_flag(SIGN, a >> 7 == 1);
    }

    fn ror_a(&mut self) {
        let mut a = self.a;
        let carry = self.get_flag(CARRY);
        self.set_flag(CARRY, (a & 0x01) == 0x01);
        a >>= 1;
        a |= if carry { 0x80 } else { 0 };
        self.a = a;
        self.set_flag(ZERO, a == 0);
        self.set_flag(SIGN, a >> 7 == 1);
    }

    fn lsr_a(&mut self) {
        let mut a = self.a;
        self.set_flag(CARRY, (a & 0x01) == 0x01);
        a >>= 1;
        self.a = a;
        self.set_flag(ZERO, a == 0);
        self.set_flag(SIGN, a >> 7 == 1);
    }

    fn rol(&mut self) {
        let mut value = self.value;
        let carry = self.get_flag(CARRY);
        self.set_flag(CARRY, (value >> 7) == 0x01);
        value <<= 1;
        value |= if carry { 1 } else { 0 };
        let addr = self.addr;
        self.write8(addr, value);
        self.set_flag(ZERO, value == 0);
        self.set_flag(SIGN, value >> 7 == 1);
    }

    fn ror(&mut self) {
        let mut value = self.value;
        let carry = self.get_flag(CARRY);
        self.set_flag(CARRY, (value & 0x01) == 0x01);
        value >>= 1;
        value |= if carry { 0x80 } else { 0 };
        let addr = self.addr;
        self.write8(addr, value);
        self.set_flag(ZERO, value == 0);
        self.set_flag(SIGN, value >> 7 == 1);
    }

    fn asl(&mut self) {
        let mut value = self.value;
        self.set_flag(CARRY, (value >> 7) == 0x01);
        value <<= 1;
        let addr = self.addr;
        self.write8(addr, value);
        self.set_flag(ZERO, value == 0);
        self.set_flag(SIGN, value >> 7 == 1);
    }

    fn lsr(&mut self) {
        let mut value = self.value;
        self.set_flag(CARRY, (value & 0x01) == 0x01);
        value >>= 1;
        let addr = self.addr;
        self.write8(addr, value);
        self.set_flag(ZERO, value == 0);
        self.set_flag(SIGN, value >> 7 == 1);
    }

    fn ora(&mut self) {
        self.a |= self.value;
        let a = self.a;
        self.set_flag(ZERO, a == 0);
        self.set_flag(SIGN, a >> 7 == 1);
    }

    fn eor(&mut self) {
        self.a ^= self.value;
        let a = self.a;
        self.set_flag(ZERO, a == 0);
        self.set_flag(SIGN, a >> 7 == 1);
    }

    fn sbc(&mut self) {
        // first check for carry
        let carry = if self.get_flag(CARRY) { 0 } else { 1 };
        let orig_a = i16::from(self.a);
        let value = i16::from(self.value);
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
        let orig_a = i16::from(self.a);
        let value = i16::from(self.value);
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

    fn sei(&mut self) {
        self.set_flag(INTERRUPT, true);
    }

    fn clc(&mut self) {
        self.set_flag(CARRY, false);
    }

    fn cli(&mut self) {
        self.set_flag(INTERRUPT, false);
    }

    fn clv(&mut self) {
        self.set_flag(OVERFLOW, false);
    }

    fn cld(&mut self) {
        self.set_flag(DECIMAL, false);
    }

    fn sed(&mut self) {
        self.set_flag(DECIMAL, true);
    }

    fn beq(&mut self) {
        if self.get_flag(ZERO) {
            self.pc = self.addr;
            self.ticks += 1;
        }
    }

    fn bmi(&mut self) {
        if self.get_flag(SIGN) {
            self.pc = self.addr;
            self.ticks += 1;
        }
    }

    fn bpl(&mut self) {
        if !self.get_flag(SIGN) {
            self.pc = self.addr;
            self.ticks += 1;
        }
    }

    fn bvc(&mut self) {
        if !self.get_flag(OVERFLOW) {
            self.pc = self.addr;
            self.ticks += 1;
        }
    }

    fn bvs(&mut self) {
        if self.get_flag(OVERFLOW) {
            self.pc = self.addr;
            self.ticks += 1;
        }
    }

    fn bne(&mut self) {
        if !self.get_flag(ZERO) {
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

    fn cmp(&mut self) {
        let a = self.a;
        let value = self.value;
        self.set_flag(CARRY, a >= value);
        self.set_flag(ZERO, a == value);
        self.set_flag(SIGN, a < value);
    }

    fn cpx(&mut self) {
        let x = self.x;
        let value = self.value;
        self.set_flag(CARRY, x >= value);
        self.set_flag(ZERO, x == value);
        self.set_flag(SIGN, x < value);
    }

    fn cpy(&mut self) {
        let y = self.y;
        let value = self.value;
        self.set_flag(CARRY, y >= value);
        self.set_flag(ZERO, y == value);
        self.set_flag(SIGN, y < value);
    }

    fn pha(&mut self) {
        let sp: u16 = 0x100 + u16::from(self.sp);
        let a = self.a;
        self.write8(sp, a);
        self.sp -= 1;
        self.ticks += 1;
    }

    fn pla(&mut self) {
        self.sp += 1;
        let sp: u16 = 0x100 + u16::from(self.sp);
        self.a = self.read8(sp);
        self.ticks += 2;
    }

    fn txs(&mut self) {
        self.sp = self.x;
    }

    fn tsx(&mut self) {
        self.x = self.sp;
    }

    fn php(&mut self) {
        let sp: u16 = 0x100 + u16::from(self.sp);
        let status = self.status;
        self.write8(sp, status);
        self.sp -= 1;
        self.ticks += 1;
    }

    fn plp(&mut self) {
        self.sp += 1;
        let sp: u16 = 0x100 + u16::from(self.sp);
        self.status = self.read8(sp);
        self.ticks += 2;
    }

    fn jsr(&mut self) {
        let sp: u16 = 0x100 + u16::from(self.sp);
        let pc = self.pc - 1;
        let pc_high = (pc >> 8) as u8;
        let pc_low = (pc & 0x00ff) as u8;
        self.write8(sp, pc_high);
        self.write8(sp - 1, pc_low);
        self.sp -= 2;

        self.pc = self.addr;
        self.ticks += 2;
    }

    fn brk(&mut self) {
        if self.code_breakpoint {
            self.requested_code_breakpoint = true;
        } else {
            self.interrupt(0xfffe);
        }
    }

    fn interrupt(&mut self, address: u16) {
        let sp: u16 = 0x100 + u16::from(self.sp);
        let pc = self.pc;
        let pc_high = (pc >> 8) as u8;
        let pc_low = (pc & 0x00ff) as u8;
        self.write8(sp, pc_high);
        self.write8(sp - 1, pc_low);
        let status = self.status;
        self.write8(sp - 2, status);
        self.sp -= 3;

        self.addr = self.read16(address);

        self.pc = self.addr;
        self.ticks += 5;
    }

    fn reset(&mut self, address: u16) {
        self.status = ALWAYS_SET | INTERRUPT;
        self.addr = self.read16(address);
        self.pc = self.addr;
        self.sp = 0xff;
        self.a = 0;
        self.x = 0;
        self.y = 0;
    }

    fn rts(&mut self) {
        self.sp += 1;
        let sp: u16 = 0x100 + u16::from(self.sp);
        let pc_low = u16::from(self.read8(sp));
        let pc_high = u16::from(self.read8(sp + 1));
        self.sp += 1;
        self.pc = (pc_high << 8 | pc_low) + 1;
        self.ticks += 4;
    }

    fn rti(&mut self) {
        self.sp += 1;
        let sp = 0x100 + u16::from(self.sp);
        let status = self.read8(sp);
        let pc_low = u16::from(self.read8(sp + 1));
        let pc_high = u16::from(self.read8(sp + 2));
        self.sp += 2;
        self.pc = pc_high << 8 | pc_low;
        self.status = status;
        self.ticks += 4;
    }

    fn nop(&mut self) {}

    fn invalid(&mut self) {
        panic!(
            "invalid opcode ${:02X} at ${:04X}",
            self.current_opcode, self.debug_pc
        );
    }
}

impl<T: AddressBusIO<u16, u8>> Clock for MOS6502<T> {
    fn step(&mut self) {
        self.debug_pc = self.pc;
        let opcode = self.read8_from_pc();
        self.current_opcode = opcode;
        self.opcode = self.opcodes[opcode as usize];
        // fetch
        (self.opcode.fetch)(self);
        // execute
        (self.opcode.fun)(self);
        if self.debug {
            let f_s = if self.get_flag(SIGN) { "S" } else { "-" };
            let f_v = if self.get_flag(OVERFLOW) { "V" } else { "-" };
            let f_z = if self.get_flag(ZERO) { "Z" } else { "-" };
            let f_c = if self.get_flag(CARRY) { "C" } else { "-" };

            self.debug_line = format!(
                "{} [A=${:02X} X=${:02X} Y=${:02X} SP=${:02X} |${:02X}| {}{}{}{}]",
                self.debug_line,
                self.a,
                self.x,
                self.y,
                self.sp,
                self.current_opcode,
                f_s,
                f_v,
                f_z,
                f_c
            );
        }
    }
}

impl<T: AddressBusIO<u16, u8>> AddressBusIO<u16, u8> for MOS6502<T> {
    fn read(&mut self, address: u16) -> u8 {
        self.read8(address)
    }

    fn write(&mut self, address: u16, data: u8) {
        self.write8(address, data)
    }
}

impl<T: AddressBusIO<u16, u8>> Debug<u16, u8> for MOS6502<T> {
    fn address_str(&self, address: u16) -> String {
        format!("${:04X}", address)
    }

    fn data_str(&self, data: u8) -> String {
        format!("${:02X}", data)
    }

    fn inspect(&mut self, address: u16) -> u8 {
        self.read(address)
    }

    fn inject(&mut self, address: u16, data: u8) {
        self.write(address, data);
    }

    fn get_cursor(&self) -> u16 {
        self.pc
    }

    fn next(&mut self) {
        self.step();
    }

    fn set_cursor(&mut self, address: u16) {
        self.pc = address;
    }

    fn set_code_breakpoint(&mut self, enable: bool) {
        self.code_breakpoint = enable;
    }

    fn is_code_breakpoint_requested(&mut self) -> bool {
        let requested = self.requested_code_breakpoint;
        self.requested_code_breakpoint = false;
        requested
    }
}

impl<T: AddressBusIO<u16, u8>> Interrupt<u16> for MOS6502<T> {
    // line 4: IRQ/BRK $FFFE/$FFFF
    // line 6: NMI $FFFA/$FFFB
    // line 40: RESET $FFFC/$FFFD
    fn raise(&mut self, line: u16) {
        match line {
            4 => {
                if !self.get_flag(INTERRUPT) && !self.get_flag(BRK) {
                    self.interrupt(0xfffe);
                    // set it later so the status can be restored from the stack
                    self.set_flag(BRK, true);
                }
            }
            6 => self.interrupt(0xfffa),
            40 => {
                if !self.get_flag(INTERRUPT) {
                    self.reset(0xfffc)
                }
            }
            _ => println!("raised interrupt on line {}", line),
        }
    }
}

#[cfg(test)]
mod tests;
