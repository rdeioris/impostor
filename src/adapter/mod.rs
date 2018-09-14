use {Address, AddressBusIO, Data};

pub struct BusAdapter<'a, T: Address, U: Data> {
    connection: &'a mut AddressBusIO<T, U>,
}

impl<'a, T: Address, U: Data> BusAdapter<'a, T, U> {
    pub fn new(bus: &'a mut AddressBusIO<T, U>) -> BusAdapter<'a, T, U> {
        BusAdapter { connection: bus }
    }
}

impl<'a, T: Address, U: Data, V: Address, Z: Data> AddressBusIO<T, U> for BusAdapter<'a, V, Z> {
    fn read(&mut self, address: T) -> U {
        U::from(self.connection.read(V::from(address).unwrap())).unwrap()
    }
    fn write(&mut self, address: T, value: U) {
        self.connection
            .write(V::from(address).unwrap(), Z::from(value).unwrap())
    }
}
