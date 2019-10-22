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
    max_size: u64,
}

impl BlockDevice {
    pub fn new(mut file: File, block_size: usize) -> BlockDevice {
        let max_size = file.seek(SeekFrom::End(0)).unwrap();
        BlockDevice {
            file,
            block_size,
            max_size,
        }
    }

    pub fn from_filename<P: AsRef<Path>>(filename: P, block_size: usize) -> BlockDevice {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(filename)
            .unwrap();
        let max_size = file.seek(SeekFrom::End(0)).unwrap();
        BlockDevice {
            file: file,
            block_size,
            max_size,
        }
    }
}

impl<T: Address + As<usize>> AddressBusBlockIO<T, u8> for BlockDevice {
    fn read(&mut self, address: T, buffer: &mut [u8]) {
        let offset = (address.as_() * self.block_size) as u64;
        if offset + self.block_size as u64 >= self.max_size {
            return;
        }
        self.file.seek(SeekFrom::Start(offset)).unwrap();
        self.file.read_exact(buffer).unwrap();
    }

    fn write(&mut self, address: T, buffer: &[u8]) {
        let offset = (address.as_() * self.block_size) as u64;
        if offset + self.block_size as u64 >= self.max_size {
            return;
        }
        self.file.seek(SeekFrom::Start(offset)).unwrap();
        self.file.write_all(buffer).unwrap();
        self.file.sync_all().unwrap();
    }
}
