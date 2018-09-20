use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use {Address, AddressBusIO, Data};

struct AddressMapping<'a, T: Address, U: Data> {
    start: T,
    end: T,
    connection: &'a mut AddressBusIO<T, U>,
}

pub struct MemoryController<'a, T: Address, U: Data> {
    mappings: Vec<AddressMapping<'a, T, U>>,
}

impl<'a, T: Address, U: Data> MemoryController<'a, T, U> {
    pub fn new() -> MemoryController<'a, T, U> {
        MemoryController {
            mappings: Vec::new(),
        }
    }

    pub fn map(&mut self, start: T, end: T, connection: &'a mut AddressBusIO<T, U>) {
        self.mappings.push(AddressMapping {
            start: start,
            end: end,
            connection: connection,
        });
    }
}

impl<'a, T: Address, U: Data> AddressBusIO<T, U> for MemoryController<'a, T, U> {
    fn read(&mut self, address: T) -> U {
        for mapping in &mut self.mappings {
            if address >= mapping.start && address <= mapping.end {
                return mapping.connection.read(address - mapping.start);
            }
        }
        U::zero()
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

struct AddressMappingBoxed<T: Address, U: Data> {
    start: T,
    end: T,
    connection: Box<AddressBusIO<T, U>>,
}

pub struct MemoryControllerBoxed<T: Address, U: Data> {
    mappings: Vec<AddressMappingBoxed<T, U>>,
}

impl<T: Address, U: Data> MemoryControllerBoxed<T, U> {
    pub fn new() -> MemoryControllerBoxed<T, U> {
        MemoryControllerBoxed {
            mappings: Vec::new(),
        }
    }

    pub fn map(&mut self, start: T, end: T, connection: Box<AddressBusIO<T, U>>) {
        self.mappings.push(AddressMappingBoxed {
            start: start,
            end: end,
            connection: connection,
        });
    }
}

impl<T: Address, U: Data> AddressBusIO<T, U> for MemoryControllerBoxed<T, U> {
    fn read(&mut self, address: T) -> U {
        for mapping in &mut self.mappings {
            if address >= mapping.start && address <= mapping.end {
                return mapping.connection.read(address - mapping.start);
            }
        }
        U::zero()
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

struct AddressMappingShared<T: Address, U: Data> {
    start: T,
    end: T,
    connection: Rc<RefCell<AddressBusIO<T, U>>>,
}

pub struct MemoryControllerShared<T: Address, U: Data> {
    mappings: Vec<AddressMappingShared<T, U>>,
}

impl<T: Address, U: Data> MemoryControllerShared<T, U> {
    pub fn new() -> MemoryControllerShared<T, U> {
        MemoryControllerShared {
            mappings: Vec::new(),
        }
    }

    pub fn map(&mut self, start: T, end: T, connection: Rc<RefCell<AddressBusIO<T, U>>>) {
        self.mappings.push(AddressMappingShared {
            start: start,
            end: end,
            connection: connection,
        });
    }
}

impl<T: Address, U: Data> AddressBusIO<T, U> for MemoryControllerShared<T, U> {
    fn read(&mut self, address: T) -> U {
        for mapping in &mut self.mappings {
            if address >= mapping.start && address <= mapping.end {
                return mapping
                    .connection
                    .borrow_mut()
                    .read(address - mapping.start);
            }
        }
        U::zero()
    }

    fn write(&mut self, address: T, value: U) {
        for mapping in &mut self.mappings {
            if address >= mapping.start && address <= mapping.end {
                mapping
                    .connection
                    .borrow_mut()
                    .write(address - mapping.start, value);
                return;
            }
        }
    }
}

struct AddressMappingThreadSafe<T: Address, U: Data> {
    start: T,
    end: T,
    connection: Arc<Mutex<AddressBusIO<T, U> + Send + Sync>>,
}

pub struct MemoryControllerThreadSafe<T: Address, U: Data> {
    mappings: Vec<AddressMappingThreadSafe<T, U>>,
}

impl<T: Address, U: Data> MemoryControllerThreadSafe<T, U> {
    pub fn new() -> MemoryControllerThreadSafe<T, U> {
        MemoryControllerThreadSafe {
            mappings: Vec::new(),
        }
    }

    pub fn map(
        &mut self,
        start: T,
        end: T,
        connection: Arc<Mutex<AddressBusIO<T, U> + Send + Sync>>,
    ) {
        self.mappings.push(AddressMappingThreadSafe {
            start: start,
            end: end,
            connection: connection,
        });
    }
}

impl<T: Address, U: Data> AddressBusIO<T, U> for MemoryControllerThreadSafe<T, U> {
    fn read(&mut self, address: T) -> U {
        for mapping in &mut self.mappings {
            if address >= mapping.start && address <= mapping.end {
                return mapping
                    .connection
                    .lock()
                    .unwrap()
                    .read(address - mapping.start);
            }
        }
        U::zero()
    }

    fn write(&mut self, address: T, value: U) {
        for mapping in &mut self.mappings {
            if address >= mapping.start && address <= mapping.end {
                mapping
                    .connection
                    .lock()
                    .unwrap()
                    .write(address - mapping.start, value);
                return;
            }
        }
    }
}
