use serde::{Deserialize, Serialize};

use super::method_to_number_converter::MethodToNumberConverter;
use super::{constants::*, number_to_string_converter::NumberToStringConverter};
use custom_error::custom_error;
use std::{num::ParseIntError, str::FromStr};

custom_error! { pub MethodNumberError
    Invalid{s:String} = "Invalid MethodNumber {s}",
}

/// A contract method number.
///
/// The `MethodNumber` is used to reference contract methods in psibase. This type
/// is a convenient handler to allow consumers to parse and convert their readable
/// names.
///
/// # Examples
///
/// You can create a `MethodNumber` from [a literal string][`&str`] with [`MethodNumber::from`]:
///
/// [`MethodNumber::from`]: From::from
///
/// ```
/// use libpsibase::MethodNumber;
/// let hello = MethodNumber::from("hello");
/// ```
#[derive(Debug, Default, PartialEq, Copy, Clone, psi_macros::Fracpack, Serialize, Deserialize)]
#[fracpack(definition_will_not_change)]
pub struct MethodNumber {
    pub value: u64,
}

impl MethodNumber {
    pub fn new(value: u64) -> Self {
        MethodNumber { value }
    }

    pub fn from_exact(s: &str) -> Result<Self, MethodNumberError> {
        let result: Self = s.into();
        if result.to_string() != s {
            return Err(MethodNumberError::Invalid { s: s.into() });
        }
        Ok(result)
    }

    fn has_valid_chars(s: &str) -> bool {
        for c in s.bytes() {
            if CHAR_TO_SYMBOL_METHOD[c as usize] == 0 {
                return false;
            }
        }

        true
    }

    fn is_hash(&self) -> bool {
        self.value & (0x01_u64 << (64 - 8)) > 0
    }

    pub fn to_hash(&self) -> String {
        let mut out = String::from("#");

        let mut r = self.value;
        for _ in 0..16 {
            let symbol = (r & 0x0f) as usize + 1;
            out.push(SYMBOL_TO_CHAR_METHOD[symbol] as char);
            r >>= 4;
        }

        out
    }

    fn parse_hash(s: &str) -> Option<u64> {
        let mut chars = s.bytes();

        let first_char = chars.next().unwrap();
        if first_char != b'#' || s.len() != 17 {
            return None;
        }

        let mut output: u64 = 0;

        let mut i = 1;

        for c in chars {
            let mut sym = (CHAR_TO_SYMBOL_METHOD[c as usize] - 1) as u64;
            sym <<= 4 * (i - 1);
            output |= sym;
            i += 1;
        }

        Some(output)
    }
}

impl From<u64> for MethodNumber {
    fn from(n: u64) -> Self {
        MethodNumber { value: n }
    }
}

impl FromStr for MethodNumber {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(MethodNumber::default());
        }

        let value = MethodNumber::parse_hash(s).unwrap_or_else(|| {
            if MethodNumber::has_valid_chars(s) {
                MethodToNumberConverter::convert(s)
            } else {
                0
            }
        });
        Ok(MethodNumber { value })
    }
}

impl From<&str> for MethodNumber {
    fn from(s: &str) -> Self {
        MethodNumber::from_str(s).unwrap()
    }
}

impl std::fmt::Display for MethodNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.value == 0 {
            return f.write_str(""); // TODO: review impl empty string
        }

        if self.is_hash() {
            f.write_str(self.to_hash().as_str())
        } else {
            f.write_str(
                NumberToStringConverter::convert(
                    self.value,
                    &MODEL_CF_METHOD,
                    &SYMBOL_TO_CHAR_METHOD,
                )
                .as_str(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_name_is_zero() {
        assert_eq!(MethodNumber::from_str("").unwrap(), MethodNumber::new(0));
    }

    #[test]
    fn any_unknown_char_returns_zero() {
        assert_eq!(MethodNumber::from_str("?").unwrap(), MethodNumber::new(0));
        assert_eq!(
            MethodNumber::from_str("what?").unwrap(),
            MethodNumber::new(0)
        );
        assert_eq!(
            MethodNumber::from_str("?what").unwrap(),
            MethodNumber::new(0)
        );
        assert_eq!(
            MethodNumber::from_str("eaorintsl?").unwrap(),
            MethodNumber::new(0)
        );
        assert_eq!(
            MethodNumber::from_str("????").unwrap(),
            MethodNumber::new(0)
        );
    }

    #[test]
    fn parses_hash_successfully() {
        assert_eq!(
            MethodNumber::from_str("#hneunophpilcroch").unwrap(),
            MethodNumber::new(13346021867974402139)
        );

        assert_eq!(
            MethodNumber::from_str("#niiutpmlecuamehe").unwrap(),
            MethodNumber::new(796603392265069093)
        )
    }

    #[test]
    fn returns_hash() {
        assert_eq!(
            MethodNumber::from_str("natasharomanoff").unwrap(),
            MethodNumber::new(796603392265069093)
        );
    }

    #[test]
    fn returns_proper_numbers_from_str() {
        assert_eq!(
            MethodNumber::from_str("a").unwrap(),
            MethodNumber::new(32783)
        );
        assert_eq!(MethodNumber::from_str("b").unwrap(), MethodNumber::new(196));
        assert_eq!(
            MethodNumber::from_str("c").unwrap(),
            MethodNumber::new(32884)
        );
        assert_eq!(
            MethodNumber::from_str("abc123").unwrap(),
            MethodNumber::new(691485674271)
        );
        assert_eq!(
            MethodNumber::from_str("spiderman").unwrap(),
            MethodNumber::new(311625498215)
        );
        assert_eq!(
            MethodNumber::from_str("brucewayne").unwrap(),
            MethodNumber::new(56488722015273161)
        );
        assert_eq!(
            MethodNumber::from_str("anthonystark").unwrap(),
            MethodNumber::new(50913722085663764)
        );
        assert_eq!(
            MethodNumber::from_str("natasharomanoff").unwrap(),
            MethodNumber::new(796603392265069093)
        );
    }

    #[test]
    fn method_number_value_to_string_is_converted_successfully() {
        let name = MethodNumber::from_str("a").unwrap();
        assert_eq!(name.value, 32783);
        assert_eq!(name.to_string(), "a");

        let name = MethodNumber::from_str("b").unwrap();
        assert_eq!(name.to_string(), "b");
        let name = MethodNumber::from(196);
        assert_eq!(name.to_string(), "b");

        let name = MethodNumber::from_str("c").unwrap();
        assert_eq!(name.to_string(), "c");
        let name = MethodNumber::from(32884);
        assert_eq!(name.to_string(), "c");

        let name = MethodNumber::from_str("spiderman").unwrap();
        assert_eq!(name.to_string(), "spiderman");
        let name = MethodNumber::from(311625498215);
        assert_eq!(name.to_string(), "spiderman");

        let name = MethodNumber::from_str("anthonystark").unwrap();
        assert_eq!(name.to_string(), "anthonystark");
        let name = MethodNumber::from(50913722085663764);
        assert_eq!(name.to_string(), "anthonystark");

        let name = MethodNumber::from_str("natasharomanoff").unwrap();
        assert_eq!(name.to_string(), "#niiutpmlecuamehe");
        let name = MethodNumber::from(796603392265069093);
        assert_eq!(name.to_string(), "#niiutpmlecuamehe");

        let name = MethodNumber::from(0);
        assert_eq!(name.to_string(), "");
    }
}
