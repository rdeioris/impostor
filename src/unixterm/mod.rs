use AddressBusIO;

pub struct UnixTerm {}

impl UnixTerm {
    pub fn new() -> UnixTerm {
        UnixTerm {}
    }
}

impl AddressBusIO<u8, u8> for UnixTerm {
    fn read(&mut self, address: u8) -> u8 {
        return 0;
    }
}

impl AddressBusIO<u16, u8> for UnixTerm {
    fn read(&mut self, address: u16) -> u8 {
        <UnixTerm as AddressBusIO<u8, u8>>::read(self, address as u8)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
