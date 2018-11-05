extern crate num_traits;
extern crate rand;

pub use num_traits::AsPrimitive as As;
use num_traits::{NumAssign, PrimInt};

use std::fmt::{Display, LowerHex, UpperHex};

pub trait Address:
    PrimInt + NumAssign + Display + LowerHex + UpperHex + Sync + Send + 'static
{
}

pub trait Data: Address {}

impl<T: PrimInt + NumAssign + Display + LowerHex + UpperHex + Sync + Send + 'static> Address for T {}
impl<T: Address> Data for T {}

pub trait AddressBusIO<T: Address, U: Data> {
    fn read(&mut self, _address: T) -> U {
        U::zero()
    }
    fn write(&mut self, _address: T, _value: U) {}
}

pub trait AddressBusBlockIO<T: Address, U: Data> {
    fn read(&mut self, address: T, buffer: &mut [U]);
    fn write(&mut self, address: T, buffer: &[U]);
}

pub trait Clock {
    fn step(&mut self);
}

pub trait Interrupt<T: Address> {
    fn raise(&mut self, _line: T);
}

pub trait Debug<T: Address, U: Data> {
    fn inspect(&mut self, _address: T) -> U;
    fn inject(&mut self, _address: T, _value: U);
    fn address_str(&self, address: T) -> String;
    fn data_str(&self, data: U) -> String;
    fn set_cursor(&mut self, address: T);
    fn get_cursor(&self) -> T;
    fn next(&mut self);
    fn set_code_breakpoint(&mut self, bool);
    fn is_code_breakpoint_requested(&mut self) -> bool;
}

pub mod adapter;
pub mod audio;
pub mod chip8;
pub mod debugger;
pub mod graphics;
pub mod input;
pub mod memcontroller;
pub mod mos6502;
pub mod ram;
pub mod random;
pub mod rom;
pub mod timer;
pub mod unixterm;
pub mod utils;
pub mod storage;

#[cfg(test)]
mod tests;
