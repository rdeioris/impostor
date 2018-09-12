use {Address, AddressBusIO, Data};

pub struct Ram<T: Data> {
    cells: Vec<T>,
}

impl<T: Data> Ram<T> {
    pub fn new(size: usize) -> Ram<T> {
        Ram {
            cells: vec![T::default(); size],
        }
    }
}

impl<T: Address, U: Data> AddressBusIO<T, U> for Ram<U> {
    fn read(&mut self, address: T) -> U {
        let address64: u64 = address.into();
        return self.cells[address64 as usize];
    }

    fn write(&mut self, address: T, value: U) {
        let address64: u64 = address.into();
        self.cells[address64 as usize] = value;
    }
}
