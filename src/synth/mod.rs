use std::sync::{Arc, Mutex};
use {Address, AddressBusIO, Data};

extern crate cpal;

use std::thread;

pub struct ChipTune<T: Address, U: Data> {
    bus: Arc<Mutex<AddressBusIO<T, U>+Send+Sync>>,
}

impl<T: Address, U: Data> ChipTune<T, U> {
    pub fn new(bus: Arc<Mutex<AddressBusIO<T, U>+Send+Sync>>) -> ChipTune<T, U> {
        let mut chip_tune = ChipTune { bus: bus };
        chip_tune.run();
        return chip_tune;
    }

    fn run(&mut self) {
        thread::spawn(|| {
            let device =
                cpal::default_output_device().expect("Failed to get default output device");
            let format = device
                .default_output_format()
                .expect("Failed to get default output format");
            let event_loop = cpal::EventLoop::new();
            let stream_id = event_loop.build_output_stream(&device, &format).unwrap();
            event_loop.play_stream(stream_id.clone());

            let sample_rate = format.sample_rate.0 as f32;
            let mut sample_clock = 0f32;

            let mut next_value = || {
                sample_clock = (sample_clock + 1.0) % sample_rate;
                (sample_clock * 440.0 * 2.0 * 3.141592 / sample_rate).sin()
            };

            event_loop.run(move |_, data| match data {
                cpal::StreamData::Output {
                    buffer: cpal::UnknownTypeOutputBuffer::U16(mut buffer),
                } => {
                    for sample in buffer.chunks_mut(format.channels as usize) {
                        let value = ((next_value() * 0.5 + 0.5) * u16::max_value() as f32) as u16;
                        for out in sample.iter_mut() {
                            *out = value;
                        }
                    }
                }

                cpal::StreamData::Output {
                    buffer: cpal::UnknownTypeOutputBuffer::I16(mut buffer),
                } => {
                    for sample in buffer.chunks_mut(format.channels as usize) {
                        let value = (next_value() * i16::max_value() as f32) as i16;
                        for out in sample.iter_mut() {
                            *out = value;
                        }
                    }
                }

                cpal::StreamData::Output {
                    buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer),
                } => {
                    for sample in buffer.chunks_mut(format.channels as usize) {
                        let value = next_value();
                        for out in sample.iter_mut() {
                            *out = value;
                        }
                    }
                }

                _ => (),
            });
        });
    }
}

impl<T: Address, U: Data> AddressBusIO<T, U> for ChipTune<T, U> {}
