use std::num::ParseIntError;
use Address;

pub fn to_number<T: Address<FromStrRadixErr = ParseIntError>>(
    string: &str,
) -> Result<T, ParseIntError> {
    if string.starts_with("0x") {
        let len = string.len();
        return T::from_str_radix(&string[2..len], 16);
    }

    if string.starts_with('$') {
        let len = string.len();
        return T::from_str_radix(&string[1..len], 16);
    }

    if string.starts_with("0b") {
        let len = string.len();
        return T::from_str_radix(&string[2..len], 2);
    }

    if string.starts_with('%') {
        let len = string.len();
        return T::from_str_radix(&string[1..len], 2);
    }

    T::from_str_radix(string, 10)
}
