use std::time::Duration;
use {Address, AddressBusIO, As, Data};

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
            wave: wave.take_duration(Duration::from_millis(125)),
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

pub struct Piano {
    sink: rodio::Sink,
    waves: [rodio::source::TakeDuration<rodio::source::SineWave>; 36],
}

impl Piano {
    pub fn new(duration: u64) -> Piano {
        let device = rodio::default_output_device().unwrap();
        let waves = [
            rodio::source::SineWave::new(440).take_duration(Duration::from_millis(duration)),
            rodio::source::SineWave::new(466).take_duration(Duration::from_millis(duration)),
            rodio::source::SineWave::new(493).take_duration(Duration::from_millis(duration)),
            rodio::source::SineWave::new(523).take_duration(Duration::from_millis(duration)),
            rodio::source::SineWave::new(554).take_duration(Duration::from_millis(duration)),
            rodio::source::SineWave::new(587).take_duration(Duration::from_millis(duration)),
            rodio::source::SineWave::new(622).take_duration(Duration::from_millis(duration)),
            rodio::source::SineWave::new(659).take_duration(Duration::from_millis(duration)),
            rodio::source::SineWave::new(698).take_duration(Duration::from_millis(duration)),
            rodio::source::SineWave::new(739).take_duration(Duration::from_millis(duration)),
            rodio::source::SineWave::new(783).take_duration(Duration::from_millis(duration)),
            rodio::source::SineWave::new(830).take_duration(Duration::from_millis(duration)),
            rodio::source::SineWave::new(880).take_duration(Duration::from_millis(duration)),
            rodio::source::SineWave::new(932).take_duration(Duration::from_millis(duration)),
            rodio::source::SineWave::new(987).take_duration(Duration::from_millis(duration)),
            rodio::source::SineWave::new(1046).take_duration(Duration::from_millis(duration)),
            rodio::source::SineWave::new(1108).take_duration(Duration::from_millis(duration)),
            rodio::source::SineWave::new(1174).take_duration(Duration::from_millis(duration)),
            rodio::source::SineWave::new(1244).take_duration(Duration::from_millis(duration)),
            rodio::source::SineWave::new(1318).take_duration(Duration::from_millis(duration)),
            rodio::source::SineWave::new(1396).take_duration(Duration::from_millis(duration)),
            rodio::source::SineWave::new(1479).take_duration(Duration::from_millis(duration)),
            rodio::source::SineWave::new(1567).take_duration(Duration::from_millis(duration)),
            rodio::source::SineWave::new(1661).take_duration(Duration::from_millis(duration)),
            rodio::source::SineWave::new(1760).take_duration(Duration::from_millis(duration)),
            rodio::source::SineWave::new(1864).take_duration(Duration::from_millis(duration)),
            rodio::source::SineWave::new(1975).take_duration(Duration::from_millis(duration)),
            rodio::source::SineWave::new(2093).take_duration(Duration::from_millis(duration)),
            rodio::source::SineWave::new(2217).take_duration(Duration::from_millis(duration)),
            rodio::source::SineWave::new(2349).take_duration(Duration::from_millis(duration)),
            rodio::source::SineWave::new(2489).take_duration(Duration::from_millis(duration)),
            rodio::source::SineWave::new(2637).take_duration(Duration::from_millis(duration)),
            rodio::source::SineWave::new(2793).take_duration(Duration::from_millis(duration)),
            rodio::source::SineWave::new(2959).take_duration(Duration::from_millis(duration)),
            rodio::source::SineWave::new(3135).take_duration(Duration::from_millis(duration)),
            rodio::source::SineWave::new(3322).take_duration(Duration::from_millis(duration)),
        ];

        Piano {
            sink: rodio::Sink::new(&device),
            waves: waves,
        }
    }
}

impl<T: Address, U: Data + As<usize>> AddressBusIO<T, U> for Piano {
    fn write(&mut self, _address: T, value: U) {
        if value.as_() >= self.waves.len() {
            return;
        }
        let wave = self.waves[value.as_()].clone();
        self.sink.append(wave);
    }
}
