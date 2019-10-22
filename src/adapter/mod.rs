use {Address, AddressBusIO, As, Data};

pub struct BusAdapter<'a, T: Address, U: Data> {
    connection: &'a mut dyn AddressBusIO<T, U>,
}

impl<'a, T: Address, U: Data> BusAdapter<'a, T, U> {
    pub fn new(bus: &'a mut dyn AddressBusIO<T, U>) -> BusAdapter<'a, T, U> {
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
mod tests;
