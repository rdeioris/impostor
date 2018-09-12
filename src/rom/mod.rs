use {Address, AddressBusIO, Data};

pub struct Rom<T: Data> {
    cells: Vec<T>,
}

impl<T: Data> Rom<T> {
    pub fn new(data: Vec<T>) -> Rom<T> {
        Rom { cells: data }
    }
}

impl<T: Address, U: Data> AddressBusIO<T, U> for Rom<U> {
    fn read(&mut self, address: T) -> U {
        let address64: u64 = address.into();
        return self.cells[address64 as usize];
    }
}
