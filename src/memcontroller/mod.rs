use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use {Address, AddressBusIO, Data};

struct AddressMapping<'a, T: Address + 'a, U: Data + 'a> {
    start: T,
    end: T,
    connection: &'a mut AddressBusIO<T, U>,
}

struct MirrorMapping<T: Address> {
    start: T,
    end: T,
    mirror: T,
}

pub struct MemoryController<'a, T: Address + 'a, U: Data + 'a> {
    mappings: Vec<AddressMapping<'a, T, U>>,
    mirrors: Vec<MirrorMapping<T>>,
    pub panic_on_no_map: bool,
}

impl<'a, T: Address, U: Data> MemoryController<'a, T, U> {
    pub fn new() -> MemoryController<'a, T, U> {
        MemoryController {
            mappings: Vec::new(),
            mirrors: Vec::new(),
            panic_on_no_map: false,
        }
    }

    pub fn map(&mut self, start: T, end: T, connection: &'a mut AddressBusIO<T, U>) {
        self.mappings.push(AddressMapping {
            start: start,
            end: end,
            connection: connection,
        });
    }

    pub fn mirror(&mut self, start: T, end: T, mirror: T) {
        self.mirrors.push(MirrorMapping { start, end, mirror });
    }
}

impl<'a, T: Address, U: Data> AddressBusIO<T, U> for MemoryController<'a, T, U> {
    fn read(&mut self, address: T) -> U {
        let mut cleaned_address = address;
        // first check for mirrors
        for mirror in &self.mirrors {
            if address >= mirror.start && address <= mirror.end {
                cleaned_address = mirror.mirror + address - mirror.start;
                break;
            }
        }
        for mapping in &mut self.mappings {
            if cleaned_address >= mapping.start && cleaned_address <= mapping.end {
                return mapping.connection.read(cleaned_address - mapping.start);
            }
        }
        if self.panic_on_no_map {
            panic!("unknown mapping ${:X}", address);
        }
        U::zero()
    }

    fn write(&mut self, address: T, value: U) {
        let mut cleaned_address = address;
        // first check for mirrors
        for mirror in &self.mirrors {
            if address >= mirror.start && address <= mirror.end {
                cleaned_address = mirror.mirror + address - mirror.start;
                break;
            }
        }
        for mapping in &mut self.mappings {
            if cleaned_address >= mapping.start && cleaned_address <= mapping.end {
                mapping
                    .connection
                    .write(cleaned_address - mapping.start, value);
                return;
            }
        }
        if self.panic_on_no_map {
            panic!("unknown mapping ${:X}", address);
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

pub struct MemoryControllerSmart<'a, T: Address + 'a, U: Data + 'a> {
    mappings: Vec<AddressMapping<'a, T, U>>,
    shared_mappings: Vec<AddressMappingShared<T, U>>,
}

impl<'a, T: Address, U: Data> MemoryControllerSmart<'a, T, U> {
    pub fn new() -> MemoryControllerSmart<'a, T, U> {
        MemoryControllerSmart {
            mappings: Vec::new(),
            shared_mappings: Vec::new(),
        }
    }
    pub fn map(&mut self, start: T, end: T, connection: &'a mut AddressBusIO<T, U>) {
        self.mappings.push(AddressMapping {
            start: start,
            end: end,
            connection: connection,
        });
    }

    pub fn map_shared(&mut self, start: T, end: T, connection: Rc<RefCell<AddressBusIO<T, U>>>) {
        self.shared_mappings.push(AddressMappingShared {
            start: start,
            end: end,
            connection: connection,
        });
    }
}

impl<'a, T: Address, U: Data> AddressBusIO<T, U> for MemoryControllerSmart<'a, T, U> {
    fn read(&mut self, address: T) -> U {
        for mapping in &mut self.mappings {
            if address >= mapping.start && address <= mapping.end {
                return mapping.connection.read(address - mapping.start);
            }
        }
        for mapping in &mut self.shared_mappings {
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
                mapping.connection.write(address - mapping.start, value);
                return;
            }
        }
        for mapping in &mut self.shared_mappings {
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
