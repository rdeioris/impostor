use {Address, AddressBusIO, Data};
use std::time::Duration;

extern crate rodio;

use audio::rodio::Source;

pub struct Beeper {
    sink: rodio::Sink,
    wave: rodio::source::TakeDuration<rodio::source::SineWave>,
}

impl Beeper {
    pub fn new(frequency: u32) -> Beeper {
        let device = rodio::default_output_device().unwrap();
        let wave = rodio::source::SineWave::new(frequency);

        Beeper {
            sink: rodio::Sink::new(&device),
            wave: wave.take_duration(Duration::from_millis(100)),
        }
    }

    pub fn beep(&self) {
        let wave = self.wave.clone();
        self.sink.append(wave);
    }
}

impl<T: Address, U: Data> AddressBusIO<T, U> for Beeper {
    fn write(&mut self, _address: T, _value: U) {
        self.beep();
    }
}
