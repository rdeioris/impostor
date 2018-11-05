use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::path::Path;

use {Address, AddressBusBlockIO, As};

pub struct BlockDevice {
    file: File,
    pub block_size: usize,
}

impl BlockDevice {
    pub fn new(file: File, block_size: usize) -> BlockDevice {
        BlockDevice { file, block_size }
    }

    pub fn from_filename<P: AsRef<Path>>(filename: P, block_size: usize) -> BlockDevice {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(filename);
        BlockDevice {
            file: file.unwrap(),
            block_size,
        }
    }
}

impl<T: Address + As<usize>> AddressBusBlockIO<T, u8> for BlockDevice {
    fn read(&mut self, address: T, buffer: &mut [u8]) {
        self.file
            .seek(SeekFrom::Start((address.as_() * self.block_size) as u64))
            .unwrap();
        self.file.read(buffer).unwrap();
    }

    fn write(&mut self, address: T, buffer: &[u8]) {
        self.file
            .seek(SeekFrom::Start((address.as_() * self.block_size) as u64))
            .unwrap();
        self.file.write(buffer).unwrap();
        self.file.sync_all().unwrap();
    }
}
