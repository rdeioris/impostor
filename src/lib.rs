pub trait Address : PartialOrd + Into<u64> {}
pub trait Data : Default + Copy {}

impl<T: PartialOrd + Into<u64>> Address for T {}
impl<T: Default + Copy> Data for T {}

pub trait AddressBusIO<T: Address, U: Data> {
    fn read(&mut self, _address: T) -> U {U::default()}
    fn write(&mut self, _address: T, _value: U) {}
}

pub trait Clock {
    fn step(&mut self);
}

pub mod ram;
pub mod unixterm;
pub mod mos6502;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
