use std::cell::RefCell;
use std::rc::Rc;
use storage::BlockDevice;
use {Address, AddressBusBlockIO, AddressBusIO, As, Clock};

pub struct DmaBlock<T: Address> {
    block_device: BlockDevice,
    bus: Rc<RefCell<AddressBusIO<T, u8>>>,
    block: T,
    blocks_to_transfer: u8,
    address: T,
    flags: u8,
    block_counter: u8,
    address_counter: u8,
}

impl<T: Address> DmaBlock<T> {
    pub fn new(block_device: BlockDevice, bus: Rc<RefCell<AddressBusIO<T, u8>>>) -> DmaBlock<T> {
        DmaBlock {
            block_device,
            bus,
            block: T::zero(),
            blocks_to_transfer: 0,
            address: T::zero(),
            flags: 0,
            block_counter: 0,
            address_counter: 0,
        }
    }
}

impl<T: Address + As<usize>> Clock for DmaBlock<T> {
    fn step(&mut self) {
        if self.blocks_to_transfer == 0 {
            return;
        }

        let current_block = self.block - T::from(self.blocks_to_transfer).unwrap();
        let address = self.address
            + ((current_block - self.block) * T::from(self.block_device.block_size).unwrap());
        let mut cache_block = vec![0; self.block_device.block_size];

        if self.flags & 0x01 == 1 {
            // bus to block
            for i in 0..self.block_device.block_size {
                cache_block[i] = self.bus.borrow_mut().read(address + T::from(i).unwrap());
            }
            self.block_device.write(current_block, &cache_block);
            self.blocks_to_transfer -= 1;
        } else {
            // block to bus
            self.block_device.read(current_block, &mut cache_block);
            for i in 0..self.block_device.block_size {
                self.bus
                    .borrow_mut()
                    .write(address + T::from(i).unwrap(), cache_block[i]);
            }
            self.blocks_to_transfer -= 1;
        }
    }
}

impl AddressBusIO<u16, u8> for DmaBlock<u16> {
    fn write(&mut self, address: u16, value: u8) {
        match address {
            0 => {
                self.block |= (value as u16) >> (8 * self.block_counter);
                self.block_counter += 1;
                if self.block_counter > 1 {
                    self.block_counter = 0;
                }
            }
            1 => {
                self.address |= (value as u16) >> (8 * self.address_counter);
                self.address_counter += 1;
                if self.address_counter > 1 {
                    self.address_counter = 0;
                }
            }
            2 => self.blocks_to_transfer = value,
            3 => self.flags = value,
            _ => (),
        }
    }

    fn read(&mut self, address: u16) -> u8 {
        match address {
            2 => self.blocks_to_transfer,
            3 => self.flags,
            _ => 0,
        }
    }
}
