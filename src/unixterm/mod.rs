use std::io::{stdin, stdout, stderr, Stdin, Stdout, Stderr};
use std::io::Write;
use std::io::Read;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use AddressBusIO;

pub struct UnixTerm {
    stdin: Stdin, 
    stdout: Stdout, 
    stderr: Stderr,

    last_stdout: u8,
    last_stderr: u8,
}

impl UnixTerm {
    pub fn new() -> UnixTerm {
        UnixTerm {stdin: stdin(), stdout: stdout(), stderr: stderr(), last_stdout: 0, last_stderr: 0}
    }
}

impl AddressBusIO<u8, u8> for UnixTerm {
    fn read(&mut self, address: u8) -> u8 {
        let now = || -> u32 {
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32
        };
        match address {
            0x01 => self.last_stdout, 
            0x02 => self.last_stderr, 
            0x04 => ((now() >> 24) & 0xff) as u8,
            0x05 => ((now() >> 16) & 0xff) as u8,
            0x06 => ((now() >> 8) & 0xff) as u8,
            0x07 => now() as u8,
            _ => 0,
        }
    }

    fn write(&mut self, address: u8, value: u8) {
        let buffer = [value; 1];
        match address {
            0x01 => { self.stdout.write(&buffer).unwrap(); self.last_stdout = value;},
            0x02 => { self.stderr.write(&buffer).unwrap(); self.last_stderr = value;},
            0x03 => { process::exit(value as i32);},
            _ => {},
        }
    }
}

impl AddressBusIO<u16, u8> for UnixTerm {
    fn read(&mut self, address: u16) -> u8 {
        <UnixTerm as AddressBusIO<u8, u8>>::read(self, address as u8)
    }
    fn write(&mut self, address: u16, value: u8) {
        <UnixTerm as AddressBusIO<u8, u8>>::write(self, address as u8, value);
    }
}

#[cfg(test)]
mod tests {
    use unixterm::UnixTerm;
    use AddressBusIO;

    #[test]
    fn write_stdout() {
        let mut term  = UnixTerm::new();
        term.write(0x01 as u8, 17 as u8);
        assert_eq!(term.read(0x01 as u8), 17 as u8);
    }

    #[test]
    fn write_stderr() {
        let mut term  = UnixTerm::new();
        term.write(0x02 as u8, 22 as u8);
        assert_eq!(term.read(0x02 as u8), 22 as u8);
    }
}
