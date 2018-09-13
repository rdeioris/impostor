use std::sync::{Arc, Mutex};
extern crate chrono;
extern crate timer;

use {Address, AddressBusIO, Data};

pub struct SimpleTimer<T> {
    counter: Arc<Mutex<T>>,
    timer: timer::Timer,
    guard: Arc<Mutex<Option<timer::Guard>>>,
}

impl<T: Data> SimpleTimer<T> {
    pub fn new() -> SimpleTimer<T> {
        SimpleTimer {
            counter: Arc::new(Mutex::new(T::zero())),
            timer: timer::Timer::new(),
            guard: Arc::new(Mutex::new(None)),
        }
    }
}

impl<T: Address> AddressBusIO<T, u8> for SimpleTimer<u8> {
    fn read(&mut self, _address: T) -> u8 {
        *self.counter.lock().unwrap()
    }

    fn write(&mut self, _address: T, data: u8) {
        println!("data = {}", data);
        *self.counter.lock().unwrap() = data;
        let counter = self.counter.clone();
        let guard = self.guard.clone();
        *self.guard.lock().unwrap() = Some(self.timer.schedule_repeating(
            chrono::Duration::milliseconds(1),
            move || {
                *counter.lock().unwrap() -= 1;
                println!("timer: {}", *counter.lock().unwrap());
                if *counter.lock().unwrap() == 0 {
                    *guard.lock().unwrap() = None;
                }
            },
        ));
    }
}
