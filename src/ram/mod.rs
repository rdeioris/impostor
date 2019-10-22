use std::cmp;
use {Address, AddressBusIO, As, Data};

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
        let length = offset + cmp::min(data.len(), self.cells.len());
        for (index, i) in (offset..length).enumerate() {
            self.cells[i] = data[index];
        }
    }
}

impl<T: Address + As<usize>, U: Data> AddressBusIO<T, U> for Ram<U> {
    fn read(&mut self, address: T) -> U {
        self.cells[address.as_()]
    }

    fn write(&mut self, address: T, value: U) {
        self.cells[address.as_()] = value;
    }
}
