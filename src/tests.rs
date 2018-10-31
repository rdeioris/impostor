use {Address, AddressBusIO, Data};

struct TestAddressBusIO<T: Address, U: Data> {
    _address: T,
    _data: U,
}
impl<T: Address, U: Data> Default for TestAddressBusIO<T, U> {
    fn default() -> TestAddressBusIO<T, U> {
        TestAddressBusIO {
            _address: T::zero(),
            _data: U::zero(),
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
