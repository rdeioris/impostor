use std::io::Read;
use std::io::Write;
use std::io::{stderr, stdin, stdout, Stderr, Stdout};
use std::process;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use AddressBusIO;

pub struct UnixTerm {
    stdout: Stdout,
    stderr: Stderr,

    last_stdout: u8,
    last_stderr: u8,

    channel_data: (Sender<u8>, Receiver<u8>),
    channel_command: Sender<u8>,
}

impl UnixTerm {
    pub fn new() -> UnixTerm {
        let channel_command = channel();

        let term = UnixTerm {
            stdout: stdout(),
            stderr: stderr(),
            last_stdout: 0,
            last_stderr: 0,
            channel_data: channel(),
            channel_command: channel_command.0.clone(),
        };

        let sender = term.channel_data.0.clone();

        thread::spawn(move || {
            let stdin = stdin();
            loop {
                match channel_command.1.recv() {
                    Ok(_) => {
                        let mut buffer = [0; 1];
                        match stdin.lock().read(&mut buffer) {
                            Ok(_) => match sender.send(buffer[0]) {
                                _ => (), // swallow errors;
                            },
                            Err(_) => panic!("stdin error"),
                        }
                    }
                    Err(_) => panic!("stdin channel error"),
                }
            }
        });

        return term;
    }
}

impl AddressBusIO<u8, u8> for UnixTerm {
    fn read(&mut self, address: u8) -> u8 {
        // wake up thread
        self.channel_command.send(0).unwrap();
        match address {
            0x00 => match self.channel_data.1.try_recv() {
                Ok(value) => value,
                Err(_) => 0,
            },
            0x01 => self.last_stdout,
            0x02 => self.last_stderr,
            _ => 0,
        }
    }

    fn write(&mut self, address: u8, value: u8) {
        let buffer = [value; 1];
        match address {
            0x01 => {
                self.stdout.write(&buffer).unwrap();
                self.stdout.flush().unwrap();
                self.last_stdout = value;
            }
            0x02 => {
                self.stderr.write(&buffer).unwrap();
                self.stderr.flush().unwrap();
                self.last_stderr = value;
            }
            0x03 => {
                process::exit(value as i32);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use unixterm::UnixTerm;
    use AddressBusIO;

    #[test]
    fn write_stdout() {
        let mut term = UnixTerm::new();
        term.write(0x01 as u8, 17 as u8);
        assert_eq!(term.read(0x01 as u8), 17 as u8);
    }

    #[test]
    fn write_stderr() {
        let mut term = UnixTerm::new();
        term.write(0x02 as u8, 22 as u8);
        assert_eq!(term.read(0x02 as u8), 22 as u8);
    }
}
