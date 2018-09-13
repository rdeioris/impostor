use {Address, AddressBusIO, Data};

pub struct BusAdapter<T: Address, U: Data> {
    connection: Box<AddressBusIO<T, U>>, 
}

impl<T: Address, U: Data> BusAdapter<T, U> {
    pub fn new(bus: Box<AddressBusIO<T, U>>) -> BusAdapter<T, U> {
       BusAdapter{connection: bus} 
    }
}

impl<T: Address, U: Data, V: Address, Z: Data> AddressBusIO<T, U> for BusAdapter<V, Z> {
    fn read(&mut self, address: T) -> U {
        U::from(self.connection.read(V::from(address).unwrap())).unwrap()
    }
    fn write(&mut self, address: T, value: U) {
        self.connection.write(V::from(address).unwrap(), Z::from(value).unwrap())
    }
}
