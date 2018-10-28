extern crate rustyline;
use self::rustyline::Editor;

use {Address, AddressBusIO, Data};

use std::num::ParseIntError;
use utils::to_number;

pub fn debugger<
    T: Address<FromStrRadixErr = ParseIntError>,
    U: Data<FromStrRadixErr = ParseIntError>,
>(
    prompt: String,
    bus: &mut AddressBusIO<T, U>,
) -> bool {
    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline(prompt.as_ref());
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_ref());
                let mut iter = line.split_whitespace();
                match iter.next() {
                    Some("p") => match iter.next() {
                        Some(value) => match to_number::<T>(value) {
                            Ok(address) => match iter.next() {
                                Some(amount) => match to_number::<T>(amount) {
                                    Ok(qt) => {
                                        let mut counter = T::zero();
                                        let mut next_line = 0;
					print!("{}: ", bus.address_str(address + counter));
                                        while counter < qt {
                                            let data = bus.read(address + counter);
                                            print!("{} ", bus.data_str(data));
                                            if next_line == 15 {
                                                println!("");
						print!("{}: ", bus.address_str(address + counter + T::one()));
                                                next_line = 0;
                                            } else {
                                                next_line += 1;
                                            }
                                            counter += T::one();
                                        }
                                        println!("");
                                    }
                                    Err(err) => println!("Error: {}", err),
                                },
                                None => println!("${:02X}", bus.read(address)),
                            },
                            Err(err) => println!("Error: {}", err),
                        },
                        _ => println!("syntax: p <address>"),
                    },
                    Some("w") => match iter.next() {
                        Some(value) => match to_number::<T>(value) {
                            Ok(address) => match iter.next() {
                                Some(argument) => match to_number::<U>(argument) {
                                    Ok(data) => {
                                        bus.write(address, data);
                                    },
                                    Err(err) => println!("Error: {}", err),
                                },
                                _ => println!("syntax: w <address> <value>"),
                            },
                            Err(err) => println!("Error: {}", err),
                        }
                        _ => println!("syntax: w <address> <value>"),
                    },
                    Some("q") => return false,
                    Some("r") => return false,
                    Some("s") => return true,
                    Some(command) => println!("unknown command {}", command),
                    None => (),
                }
            }
            Err(err) => {
                println!("Error: {}", err);
                return true;
            }
        }
    }
}
