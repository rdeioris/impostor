use num_traits::Num;
use std::num::ParseIntError;

pub fn to_number<T: Num<FromStrRadixErr = ParseIntError>>(
    string: &str,
) -> Result<T, ParseIntError> {
    if string.starts_with("0x") {
        let len = string.len();
        return T::from_str_radix(&string[2..len], 16);
    }

    if string.starts_with("0b") {
        let len = string.len();
        return T::from_str_radix(&string[2..len], 2);
    }

    return T::from_str_radix(string, 10);
}
