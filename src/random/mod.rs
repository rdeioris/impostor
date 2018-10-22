use rand;
use rand::distributions::{Distribution, Standard};
use {Address, AddressBusIO, Data};

pub struct Random<T: Data> {
    value: T,
}

impl<T: Data> Random<T> {
    pub fn new() -> Random<T> {
        Random { value: T::zero() }
    }
}

impl<T: Address, U: Data> AddressBusIO<T, U> for Random<U>
where
    Standard: Distribution<U>,
{
    fn read(&mut self, _address: T) -> U {
        return self.value;
    }

    fn write(&mut self, _address: T, _value: U) {
        self.value = rand::random::<U>();
    }
}
