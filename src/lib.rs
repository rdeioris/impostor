pub trait Address: PartialOrd + PartialEq + Into<u64> + Copy + Default {}
pub trait Data: Address {}

impl<T: PartialOrd + PartialEq + Into<u64> + Copy + Default> Address for T {}
impl<T: Address> Data for T {}

pub trait AddressBusIO<T: Address, U: Data> {
    fn read(&mut self, _address: T) -> U {
        U::default()
    }
    fn write(&mut self, _address: T, _value: U) {}
}

pub trait Clock {
    fn step(&mut self);
}

pub mod memcontroller;
pub mod mos6502;
pub mod ram;
pub mod rom;
pub mod unixterm;

#[cfg(test)]
mod tests {
    use {AddressBusIO, Address, Data};
    struct TestAddressBusIO<T: Address, U: Data> { address: T, data: U}
    impl<T: Address, U: Data> Default for TestAddressBusIO<T, U> {
        fn default() -> TestAddressBusIO<T, U> {
            TestAddressBusIO{address: T::default(), data: U::default()}
        }
    }
    impl<T: Address, U: Data> AddressBusIO<T, U> for TestAddressBusIO<T, U> {}

    #[test]
    fn address_bus_io_u8_u8() {
        let mut bus : TestAddressBusIO<u8, u8> = TestAddressBusIO::default();
        bus.write(0xff, 0);
        assert_eq!(bus.read(0xff), 0);
    }

    #[test]
    fn address_bus_io_u16_u8() {
        let mut bus : TestAddressBusIO<u16, u8> = TestAddressBusIO::default();
        bus.write(0xff, 0);
        assert_eq!(bus.read(0xffff), 0);
    }

    #[test]
    fn address_bus_io_u32_u8() {
        let mut bus : TestAddressBusIO<u32, u8> = TestAddressBusIO::default();
        bus.write(0xffffffff, 0);
        assert_eq!(bus.read(0xffffffff), 0);
    }

    #[test]
    fn address_bus_io_u64_u8() {
        let mut bus : TestAddressBusIO<u64, u8> = TestAddressBusIO::default();
        bus.write(0xffffffffffffffff, 0);
        assert_eq!(bus.read(0xffffffffffffffff), 0);
    }
}
