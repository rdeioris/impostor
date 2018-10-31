use mos6502::{CARRY, MOS6502, SIGN, ZERO};
use ram::Ram;
use AddressBusIO;
use Clock;

#[test]
fn test_adc_immediate() {
    let mut ram = Ram::new(1024);
    ram.fill(vec![0x69, 0x01], 0);
    let mut cpu = MOS6502::new(ram);
    cpu.a = 1;
    cpu.step();
    assert_eq!(cpu.a, 2);
    assert_eq!(cpu.get_flag(CARRY), false);
    assert_eq!(cpu.get_flag(ZERO), false);
}

#[test]
fn test_asl_accumulator() {
    let mut ram = Ram::new(1024);
    ram.fill(vec![0x0a], 0);
    let mut cpu = MOS6502::new(ram);
    cpu.a = 2;
    cpu.step();
    assert_eq!(cpu.a, 4);
    assert_eq!(cpu.get_flag(CARRY), false);
    assert_eq!(cpu.get_flag(ZERO), false);
}

#[test]
fn test_asl_accumulator_carry() {
    let mut ram = Ram::new(1024);
    ram.fill(vec![0x0a], 0);
    let mut cpu = MOS6502::new(ram);
    cpu.a = 0x80;
    cpu.step();
    assert_eq!(cpu.a, 0);
    assert_eq!(cpu.get_flag(CARRY), true);
    assert_eq!(cpu.get_flag(ZERO), true);
}

#[test]
fn test_asl_accumulator_sign() {
    let mut ram = Ram::new(1024);
    ram.fill(vec![0x0a], 0);
    let mut cpu = MOS6502::new(ram);
    cpu.a = 0x40;
    cpu.step();
    assert_eq!(cpu.a, 0x80);
    assert_eq!(cpu.get_flag(CARRY), false);
    assert_eq!(cpu.get_flag(ZERO), false);
    assert_eq!(cpu.get_flag(SIGN), true);
}

#[test]
fn test_asl_zero_page() {
    let mut ram = Ram::new(1024);
    ram.fill(vec![0x06, 0x0f], 0);
    ram.fill(vec![0x04], 0x000f);
    let mut cpu = MOS6502::new(ram);
    cpu.step();
    assert_eq!(cpu.read(0x000f), 8);
    assert_eq!(cpu.get_flag(CARRY), false);
    assert_eq!(cpu.get_flag(ZERO), false);
}

#[test]
fn test_asl_absolute_carry() {
    let mut ram = Ram::new(1024);
    ram.fill(vec![0x0e, 0x01, 0x02], 0);
    ram.fill(vec![0xff], 0x0201);
    let mut cpu = MOS6502::new(ram);
    cpu.step();
    assert_eq!(cpu.read(0x0201), 0xfe);
    assert_eq!(cpu.get_flag(CARRY), true);
    assert_eq!(cpu.get_flag(ZERO), false);
}

#[test]
fn test_lsr_accumulator() {
    let mut ram = Ram::new(1024);
    ram.fill(vec![0x4a], 0);
    let mut cpu = MOS6502::new(ram);
    cpu.a = 2;
    cpu.step();
    assert_eq!(cpu.a, 1);
    assert_eq!(cpu.get_flag(CARRY), false);
    assert_eq!(cpu.get_flag(ZERO), false);
}

#[test]
fn test_lsr_accumulator_carry() {
    let mut ram = Ram::new(1024);
    ram.fill(vec![0x4a], 0);
    let mut cpu = MOS6502::new(ram);
    cpu.a = 1;
    cpu.step();
    assert_eq!(cpu.a, 0);
    assert_eq!(cpu.get_flag(CARRY), true);
    assert_eq!(cpu.get_flag(ZERO), true);
}

#[test]
fn test_lsr_absolute_carry() {
    let mut ram = Ram::new(1024);
    ram.fill(vec![0x4e, 0x02, 0x03], 0);
    ram.fill(vec![0xff], 0x0302);
    let mut cpu = MOS6502::new(ram);
    cpu.step();
    assert_eq!(cpu.read(0x0302), 0x7f);
    assert_eq!(cpu.get_flag(CARRY), true);
    assert_eq!(cpu.get_flag(ZERO), false);
}

#[test]
fn test_rol_accumulator_carry() {
    let mut ram = Ram::new(1024);
    ram.fill(vec![0x2a], 0);
    let mut cpu = MOS6502::new(ram);
    cpu.set_flag(CARRY, true);
    cpu.a = 0x80;
    cpu.step();
    assert_eq!(cpu.a, 1);
    assert_eq!(cpu.get_flag(CARRY), true);
    assert_eq!(cpu.get_flag(ZERO), false);
    assert_eq!(cpu.get_flag(SIGN), false);
}

#[test]
fn test_rol_accumulator_no_carry() {
    let mut ram = Ram::new(1024);
    ram.fill(vec![0x2a], 0);
    let mut cpu = MOS6502::new(ram);
    cpu.a = 0x80;
    cpu.step();
    assert_eq!(cpu.a, 0);
    assert_eq!(cpu.get_flag(CARRY), true);
    assert_eq!(cpu.get_flag(ZERO), true);
    assert_eq!(cpu.get_flag(SIGN), false);
}

#[test]
fn test_rol_accumulator_no_carry_sign() {
    let mut ram = Ram::new(1024);
    ram.fill(vec![0x2a], 0);
    let mut cpu = MOS6502::new(ram);
    cpu.a = 0x40;
    cpu.step();
    assert_eq!(cpu.a, 0x80);
    assert_eq!(cpu.get_flag(CARRY), false);
    assert_eq!(cpu.get_flag(ZERO), false);
    assert_eq!(cpu.get_flag(SIGN), true);
}

#[test]
fn test_rol_zero_page_carry() {
    let mut ram = Ram::new(1024);
    ram.fill(vec![0x26, 0x02, 0x80], 0);
    let mut cpu = MOS6502::new(ram);
    cpu.set_flag(CARRY, true);
    cpu.step();
    assert_eq!(cpu.read(0x0002), 1);
    assert_eq!(cpu.get_flag(CARRY), true);
    assert_eq!(cpu.get_flag(ZERO), false);
    assert_eq!(cpu.get_flag(SIGN), false);
}

#[test]
fn test_ror_zero_page_carry() {
    let mut ram = Ram::new(1024);
    ram.fill(vec![0x66, 0x02, 0x81], 0);
    let mut cpu = MOS6502::new(ram);
    cpu.set_flag(CARRY, true);
    cpu.step();
    assert_eq!(cpu.read(0x0002), 0xc0);
    assert_eq!(cpu.get_flag(CARRY), true);
    assert_eq!(cpu.get_flag(ZERO), false);
    assert_eq!(cpu.get_flag(SIGN), true);
}

#[test]
fn test_ror_zero_page_no_carry() {
    let mut ram = Ram::new(1024);
    ram.fill(vec![0x66, 0x02, 0x80], 0);
    let mut cpu = MOS6502::new(ram);
    cpu.step();
    assert_eq!(cpu.read(0x0002), 0x40);
    assert_eq!(cpu.get_flag(CARRY), false);
    assert_eq!(cpu.get_flag(ZERO), false);
    assert_eq!(cpu.get_flag(SIGN), false);
}

#[test]
fn test_ror_accumulator_no_carry() {
    let mut ram = Ram::new(1024);
    ram.fill(vec![0x6a], 0);
    let mut cpu = MOS6502::new(ram);
    cpu.a = 0x40;
    cpu.step();
    assert_eq!(cpu.a, 0x20);
    assert_eq!(cpu.get_flag(CARRY), false);
    assert_eq!(cpu.get_flag(ZERO), false);
    assert_eq!(cpu.get_flag(SIGN), false);
}

#[test]
fn test_ror_accumulator_carry() {
    let mut ram = Ram::new(1024);
    ram.fill(vec![0x6a], 0);
    let mut cpu = MOS6502::new(ram);
    cpu.set_flag(CARRY, true);
    cpu.a = 0x40;
    cpu.step();
    assert_eq!(cpu.a, 0xa0);
    assert_eq!(cpu.get_flag(CARRY), false);
    assert_eq!(cpu.get_flag(ZERO), false);
    assert_eq!(cpu.get_flag(SIGN), true);
}
