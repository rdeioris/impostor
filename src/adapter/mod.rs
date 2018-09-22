use {Address, AddressBusIO, As, Data};

pub struct BusAdapter<'a, T: Address+'a, U: Data+'a> {
    connection: &'a mut AddressBusIO<T, U>,
}

impl<'a, T: Address, U: Data> BusAdapter<'a, T, U> {
    pub fn new(bus: &'a mut AddressBusIO<T, U>) -> BusAdapter<'a, T, U> {
        BusAdapter { connection: bus }
    }
}

impl<'a, T: Address + As<V>, U: Data + As<Z>, V: Address + As<T>, Z: Data + As<U>>
    AddressBusIO<T, U> for BusAdapter<'a, V, Z>
{
    fn read(&mut self, address: T) -> U {
        self.connection.read(address.as_()).as_()
    }
    fn write(&mut self, address: T, value: U) {
        self.connection.write(address.as_(), value.as_())
    }
}

#[cfg(test)]
mod tests {
    use adapter::BusAdapter;
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
    impl<T: Address, U: Data> AddressBusIO<T, U> for TestAddressBusIO<T, U> {
        fn read(&mut self, address: T) -> U {
            U::one()
        }
    }

    #[test]
    fn converto_to_lower() {
        let mut bus: TestAddressBusIO<u8, u8> = TestAddressBusIO::default();
        let mut adapter = BusAdapter::new(&mut bus);
        assert_eq!(<AddressBusIO<u32, u32>>::read(&mut adapter, 0xaabbccdd), 1);
    }
}
