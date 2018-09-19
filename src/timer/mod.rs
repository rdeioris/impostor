use std::sync::{Arc, Mutex};
extern crate chrono;
extern crate timer;

use {Address, AddressBusIO, Data, Interrupt};

pub struct SimpleTimer<T: Data, U: Address> {
    counter: Arc<Mutex<T>>,
    timer: Arc<Mutex<timer::Timer>>,
    guard: Arc<Mutex<Option<timer::Guard>>>,
    interrupt: Arc<Mutex<Option<Arc<Mutex<Interrupt<U>>>>>>,
    interrupt_line: Arc<Mutex<U>>,
}

impl<T: Data, U: Address> SimpleTimer<T, U> {
    pub fn new() -> SimpleTimer<T, U> {
        SimpleTimer {
            counter: Arc::new(Mutex::new(T::zero())),
            timer: Arc::new(Mutex::new(timer::Timer::new())),
            guard: Arc::new(Mutex::new(None)),
            interrupt: Arc::new(Mutex::new(None)),
            interrupt_line: Arc::new(Mutex::new(U::zero())),
        }
    }

    pub fn connect_to_interrupt_line(&mut self, interrupt: Arc<Mutex<Interrupt<U>>>, line: U) {
        *self.interrupt.lock().unwrap() = Some(interrupt);
        *self.interrupt_line.lock().unwrap() = line;
    }
}

impl<T: Address, U: Data> AddressBusIO<T, U> for SimpleTimer<U, T> {
    fn read(&mut self, _address: T) -> U {
        *self.counter.lock().unwrap()
    }

    fn write(&mut self, _address: T, data: U) {
        *self.counter.lock().unwrap() = data;

        let counter = self.counter.clone();
        let guard = self.guard.clone();
        let interrupt = self.interrupt.clone();
        let interrupt_line = self.interrupt_line.clone();

        *self.guard.lock().unwrap() = Some(self.timer.lock().unwrap().schedule_repeating(
            chrono::Duration::milliseconds(1),
            move || {
                *counter.lock().unwrap() -= U::one();
                if *counter.lock().unwrap() == U::zero() {
                    *guard.lock().unwrap() = None;
                    let connection = &mut *interrupt.lock().unwrap();
                    match connection {
                        Some(peer) => peer.lock().unwrap().raise(*interrupt_line.lock().unwrap()),
                        None => (),
                    }
                }
            },
        ));
    }
}
