use std::cmp;
use {Address, AddressBusIO, Data};

pub struct Ram<T: Data> {
    cells: Vec<T>,
}

impl<T: Data> Ram<T> {
    pub fn new(size: usize) -> Ram<T> {
        Ram {
            cells: vec![T::zero(); size],
        }
    }

    pub fn fill(&mut self, data: Vec<T>, offset: usize) {
        let length = cmp::min(data.len(), self.cells.len());
        for (index, i) in (offset..length).enumerate() {
            self.cells[i] = data[index];
        }
    }
}

impl<T: Address, U: Data> AddressBusIO<T, U> for Ram<U> {
    fn read(&mut self, address: T) -> U {
        return self.cells[address.to_u64().unwrap() as usize];
    }

    fn write(&mut self, address: T, value: U) {
        self.cells[address.to_u64().unwrap() as usize] = value;
    }
}
