use std::ops::Sub;

use {Address, AddressBusIO, Data};

struct AddressMapping<T: Address, U: Data> {
    start: T,
    end: T,
    connection: Box<AddressBusIO<T, U>>,
}

pub struct MemoryController<T: Address, U: Data> {
    mappings: Vec<AddressMapping<T, U>>,
}

impl<T: Address, U: Data> MemoryController<T, U> {
    pub fn new() -> MemoryController<T, U> {
        MemoryController {
            mappings: Vec::new(),
        }
    }

    pub fn map(&mut self, start: T, end: T, connection: Box<AddressBusIO<T, U>>) {
        self.mappings.push(AddressMapping {
            start: start,
            end: end,
            connection: connection,
        });
    }
}

impl<T: Address + Sub<Output=T>, U: Data> AddressBusIO<T, U> for MemoryController<T, U> {
    fn read(&mut self, address: T) -> U {
        for mapping in &mut self.mappings {
            if address >= mapping.start && address <= mapping.end {
                return mapping.connection.read(address - mapping.start);
            }
        }
        U::default()
    }

    fn write(&mut self, address: T, value: U) {
        for mapping in &mut self.mappings {
            if address >= mapping.start && address <= mapping.end {
                mapping.connection.write(address - mapping.start, value);
                return;
            }
        }
    }
}
