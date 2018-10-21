use {AddressBusIO, Clock, Interrupt};

const CARRY: u8 = 0x10;
const HALF: u8 = 0x20;
const SUBTRACT: u8 = 0x40;
const ZERO: u8 = 0x80;

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

pub struct Z80<T: AddressBusIO<u16, u8>> {
    bus: T,

    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub flags: u8,
    pub h: u8,
    pub l: u8,

    pub pc: u16,
    pub sp: u16,

    pub debug: bool,
    pub debug_line: String,
    pub debug_pc: u16,

    pub ticks: u64,

    value: u8,
    addr: u16,

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

            debug: false,

            status: ALWAYS_SET|INTERRUPT,

            bus: bus,
        };

        opcode!(cpu. 0x00, nop)
        opcode!(cpu, 0x05, dec, b);
        opcode!(cpu, 0x64, ld, h, l);  
        opcode!(cpu, 0x76, halt);  

        return cpu;
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
        let low = self.read8(addr) as u16;
        let high = self.read8(addr + 1) as u16;
        return (high << 8) | low;
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

    fn ld(&mut self, value: u8) -> u8 {
        value
    }

    fn inc(&mut self, value: u8) {
        value+= 1
    }

    fn dec(&mut self, value: u8) {
        value+= 1
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
    }
}

impl<T: AddressBusIO<u16, u8>+Sync+Send> Interrupt<u16> for MOS6502<T> {
    fn raise(&mut self, line: u16) {
        println!("raise {}", line);
        match line {
            0x04 => if !self.get_flag(INTERRUPT) { let jmp_addr = self.read16(0xfffe) ; println!("BRK to {:04X}", jmp_addr) },
            _ => println!("raised interrupt on line {}", line),
        }
    }
}
