use std::sync::{Arc, Mutex};
extern crate chrono;
extern crate timer;

use {Address, AddressBusIO, Data};

pub struct SimpleTimer<T> {
    counter: Arc<Mutex<T>>,
    timer: timer::Timer,
    guard: Option<timer::Guard>,
}

impl<T: Data> SimpleTimer<T> {
    pub fn new() -> SimpleTimer<T> {
        SimpleTimer {
            counter: Arc::new(Mutex::new(T::zero())),
            timer: timer::Timer::new(),
            guard: None,
        }
    }
}

impl<T: Address, U: Data+Send> AddressBusIO<T, U> for SimpleTimer<U> {
    fn read(&mut self, _address: T) -> U {
        *self.counter.lock().unwrap()
    }

    fn write(&mut self, _address: T, data: U) {
        *self.counter.lock().unwrap() = data;
        let counter = self.counter.clone();
/*
        self.guard = Some(self.timer.schedule_repeating(
            chrono::Duration::milliseconds(1),
            move || {
                *counter.lock().unwrap() -= U::one();
            },
        ));
*/
    }
}
