use {Address, AddressBusIO, As, Data};

pub struct Rom<T: Data> {
    cells: Vec<T>,
}

impl<T: Data> Rom<T> {
    pub fn new(data: Vec<T>) -> Rom<T> {
        Rom { cells: data }
    }
}

impl<T: Address + As<usize>, U: Data> AddressBusIO<T, U> for Rom<U> {
    fn read(&mut self, address: T) -> U {
        return self.cells[address.as_()];
    }
}
