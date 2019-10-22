use adapter::BusAdapter;
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
impl<T: Address, U: Data> AddressBusIO<T, U> for TestAddressBusIO<T, U> {
    fn read(&mut self, _address: T) -> U {
        U::one()
    }
}

#[test]
fn converto_to_lower() {
    let mut bus: TestAddressBusIO<u8, u8> = TestAddressBusIO::default();
    let mut adapter = BusAdapter::new(&mut bus);
    assert_eq!(
        <dyn AddressBusIO<u32, u32>>::read(&mut adapter, 0xaabbccdd),
        1
    );
}
