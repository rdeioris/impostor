extern crate num_traits;

use num_traits::{NumAssign, PrimInt};
pub use num_traits::AsPrimitive as As;

pub trait Address:
    PrimInt + NumAssign + Sync + Send + 'static
{
}

pub trait Data: Address {}

impl<T: PrimInt + NumAssign + Sync + Send + 'static> Address for T {}
impl<T: Address> Data for T {}

pub trait AddressBusIO<T: Address, U: Data> {
    fn read(&mut self, _address: T) -> U {
        U::zero()
    }
    fn write(&mut self, _address: T, _value: U) {}
}

pub trait Clock {
    fn step(&mut self);
}

pub trait Interrupt<T: Address> : Sync + Send {
    fn raise(&mut self, _line: T);
}

pub mod adapter;
pub mod memcontroller;
pub mod mos6502;
pub mod chip8;
pub mod ram;
pub mod rom;
pub mod synth;
pub mod timer;
pub mod unixterm;
pub mod screen;

#[cfg(test)]
mod tests {
    use {Address, AddressBusIO, Data};
    struct TestAddressBusIO<T: Address, U: Data> {
        address: T,
        data: U,
    }
    impl<T: Address, U: Data> Default for TestAddressBusIO<T, U> {
        fn default() -> TestAddressBusIO<T, U> {
            TestAddressBusIO {
                address: T::zero(),
                data: U::zero(),
            }
        }
    }
    impl<T: Address, U: Data> AddressBusIO<T, U> for TestAddressBusIO<T, U> {}

    #[test]
    fn address_bus_io_u8_u8() {
        let mut bus: TestAddressBusIO<u8, u8> = TestAddressBusIO::default();
        bus.write(0xff, 0xff);
        assert_eq!(bus.read(0xff), 0);
    }

    #[test]
    fn address_bus_io_u16_u8() {
        let mut bus: TestAddressBusIO<u16, u8> = TestAddressBusIO::default();
        bus.write(0xff, 0xff);
        assert_eq!(bus.read(0xffff), 0);
    }

    #[test]
    fn address_bus_io_u32_u8() {
        let mut bus: TestAddressBusIO<u32, u8> = TestAddressBusIO::default();
        bus.write(0xffffffff, 0xff);
        assert_eq!(bus.read(0xffffffff), 0);
    }

    #[test]
    fn address_bus_io_u64_u8() {
        let mut bus: TestAddressBusIO<u64, u8> = TestAddressBusIO::default();
        bus.write(0xffffffffffffffff, 0xff);
        assert_eq!(bus.read(0xffffffffffffffff), 0);
    }

    #[test]
    fn address_bus_io_u8_u16() {
        let mut bus: TestAddressBusIO<u8, u16> = TestAddressBusIO::default();
        bus.write(0xff, 0xffff);
        assert_eq!(bus.read(0xff), 0);
    }

    #[test]
    fn address_bus_io_u16_u32() {
        let mut bus: TestAddressBusIO<u16, u32> = TestAddressBusIO::default();
        bus.write(0xffff, 0xaabbccdd);
        assert_eq!(bus.read(0xabcd), 0);
    }

    #[test]
    fn address_bus_io_u64_u32() {
        let mut bus: TestAddressBusIO<u64, u32> = TestAddressBusIO::default();
        bus.write(0xffffffffaabbccdd, 0xaabbccdd);
        assert_eq!(bus.read(0xaabbccddffaaffbb), 0);
    }
}
