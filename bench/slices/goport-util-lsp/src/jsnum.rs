//! JavaScript-compatible `Number` operations and pseudo-bigint handling. This
//! was the `ts_jsnum` crate.

use std::{fmt, ops};


pub const MAX_SAFE_INTEGER: Number = Number(9_007_199_254_740_991.0);
pub const MIN_SAFE_INTEGER: Number = Number(-9_007_199_254_740_991.0);

#[must_use]
pub fn nan() -> Number {
    Number::nan()
}

#[must_use]
pub fn infinity(sign: i32) -> Number {
    Number::infinity(sign)
}


/// A JavaScript IEEE-754 `Number` value.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Number(pub f64);

impl Number {
    #[must_use]
    pub const fn new(value: f64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }

    #[must_use]
    pub fn nan() -> Self {
        Self(f64::NAN)
    }

    #[must_use]
    pub fn infinity(sign: i32) -> Self {
        Self(if sign < 0 {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        })
    }

    #[must_use]
    pub fn is_nan(self) -> bool {
        self.0.is_nan()
    }

    #[must_use]
    pub fn is_infinite(self) -> bool {
        self.0.is_infinite()
    }










    #[must_use]
    pub fn floor(self) -> Self {
        Self(self.0.floor())
    }

    #[must_use]
    pub fn abs(self) -> Self {
        Self(self.0.abs())
    }

    /// ECMAScript Number remainder operation.
    #[must_use]
    pub fn remainder(self, divisor: Self) -> Self {
        match () {
            () if self.is_nan() || divisor.is_nan() || self.is_infinite() => Self::nan(),
            () if divisor.is_infinite() => self,
            () if divisor.0 == 0.0 => Self::nan(),
            () if self.0 == 0.0 => self,
            () => Self(self.0 % divisor.0),
        }
    }


}


impl From<f64> for Number {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl From<Number> for f64 {
    fn from(value: Number) -> Self {
        value.0
    }
}

macro_rules! arithmetic {
    ($trait:ident, $method:ident, $operator:tt) => {
        impl ops::$trait for Number {
            type Output = Self;

            fn $method(self, other: Self) -> Self {
                Self(self.0 $operator other.0)
            }
        }
    };
}

arithmetic!(Add, add, +);
arithmetic!(Sub, sub, -);
arithmetic!(Mul, mul, *);
arithmetic!(Div, div, /);

impl ops::Neg for Number {
    type Output = Self;

    fn neg(self) -> Self {
        Self(-self.0)
    }
}

impl ops::Rem for Number {
    type Output = Self;

    fn rem(self, other: Self) -> Self {
        self.remainder(other)
    }
}




fn is_number_character(character: char) -> bool {
    character.is_ascii_digit()
        || matches!(
            character,
            'a'..='f' | 'A'..='F' | '.' | '-' | '+' | 'x' | 'X' | 'o' | 'O'
        )
}

fn is_js_whitespace(character: char) -> bool {
    matches!(
        character,
        '\n' | '\r'
            | '\t'
            | '\u{000b}'
            | '\u{000c}'
            | '\u{0020}'
            | '\u{00a0}'
            | '\u{1680}'
            | '\u{2000}'
            ..='\u{200a}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202f}'
                | '\u{205f}'
                | '\u{3000}'
                | '\u{feff}'
    )
}

/// A decimal representation of a JavaScript bigint without requiring bigint
/// arithmetic at call sites.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PseudoBigInt {
    pub negative: bool,
    pub base10_value: String,
}

impl PseudoBigInt {

    #[must_use]
    pub fn sign(&self) -> i8 {
        if self.base10_value.is_empty() {
            0
        } else if self.negative {
            -1
        } else {
            1
        }
    }

    // Go: jsnum/pseudobigint.go:41 PseudoBigInt.Compare (ts#64280)
    /// Compares two bigints by value: -1, 0 or 1.
    #[must_use]
    pub fn compare(&self, other: &PseudoBigInt) -> i32 {
        let c = self.sign().cmp(&other.sign()) as i32;
        if c != 0 {
            return c;
        }
        // PORT: Go `len` is the byte length; the value is ASCII digits.
        let mut c = self.base10_value.len().cmp(&other.base10_value.len()) as i32;
        if c == 0 {
            c = self.base10_value.as_str().cmp(other.base10_value.as_str()) as i32;
        }
        if self.negative {
            c = -c;
        }
        c
    }

}

impl fmt::Display for PseudoBigInt {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.base10_value.is_empty() {
            formatter.write_str("0")
        } else if self.negative {
            write!(formatter, "-{}", self.base10_value)
        } else {
            formatter.write_str(&self.base10_value)
        }
    }
}


#[cfg(test)]
mod tests {
    use super::{MAX_SAFE_INTEGER, MIN_SAFE_INTEGER, Number, PseudoBigInt, parse_pseudo_big_int};

    fn assert_number(actual: Number, expected: Number) {
        if expected.is_nan() {
            assert!(actual.is_nan());
        } else {
            assert_eq!(actual.0.to_bits(), expected.0.to_bits());
        }
    }

    #[test]
    fn converts_to_int32_and_performs_bitwise_operations() {
        let cases = [
            (Number(0.0), 0),
            (Number::nan(), 0),
            (Number::infinity(1), 0),
            (Number(2_147_483_648.0), i32::MIN),
            (MAX_SAFE_INTEGER, -1),
            (MIN_SAFE_INTEGER, 1),
            (Number(-0.4), 0),
            (Number(f64::MAX), 0),
        ];
        for (input, expected) in cases {
            assert_eq!(input.to_int32(), expected);
        }
        assert_eq!(
            Number(f64::from(0xdead_beef_u32)).bitwise_not(),
            Number(559_038_736.0)
        );
        assert_eq!(
            Number(-1.0).unsigned_right_shift(Number(0.0)),
            Number(4_294_967_295.0)
        );
        assert_eq!(Number(1.0).left_shift(Number(33.0)), Number(2.0));
    }

    #[test]
    fn implements_remainder_and_exponentiation_special_cases() {
        assert!(Number(1.0).remainder(Number(0.0)).is_nan());
        assert!(Number::infinity(1).remainder(Number(2.0)).is_nan());
        assert_eq!(Number(2.0).remainder(Number::infinity(1)), Number(2.0));
        assert_eq!(
            Number(-0.0).remainder(Number(3.0)).0.to_bits(),
            (-0.0_f64).to_bits()
        );
        assert!(Number(-1.0).exponentiate(Number::infinity(1)).is_nan());
        assert!(Number(1.0).exponentiate(Number::nan()).is_nan());
        assert_eq!(
            Number(2.0).exponentiate(Number(60.0)),
            Number(1_152_921_504_606_846_976.0)
        );
    }

    #[test]
    fn formats_numbers_like_javascript() {
        let cases = [
            (Number::nan(), "NaN"),
            (Number::infinity(1), "Infinity"),
            (Number::infinity(-1), "-Infinity"),
            (Number(-0.0), "0"),
            (Number(std::f64::consts::PI), "3.141592653589793"),
            (Number(1e20), "100000000000000000000"),
            (Number(1e21), "1e+21"),
            (Number::from(f64::from_bits(1)), "5e-324"),
            (
                Number::from(f64::from_bits(0x000f_ffff_ffff_ffff)),
                "2.225073858507201e-308",
            ),
            (Number(-2.109_808_898_695_963e16), "-21098088986959630"),
            (Number(19_686_109_595_169_230_000.0), "19686109595169230000"),
        ];
        for (number, expected) in cases {
            assert_eq!(number.to_string(), expected);
        }
    }

    #[test]
    fn parses_javascript_number_strings() {
        let cases = [
            ("", Number(0.0)),
            ("1.", Number(1.0)),
            (".0000", Number(0.0)),
            ("  +Infinity\n", Number::infinity(1)),
            ("-.0", Number(-0.0)),
            ("0b1010", Number(10.0)),
            ("0o12", Number(10.0)),
            ("0x123456789abcdef0", Number(1_311_768_467_463_790_300.0)),
            ("1e1000", Number::infinity(1)),
            (
                "010000000000000000000",
                Number(10_000_000_000_000_000_000.0),
            ),
            ("\u{00a0}1234.56\u{feff}", Number(1234.56)),
        ];
        for (text, expected) in cases {
            assert_number(Number::from_string(text), expected);
        }
        for invalid in ["NaN", ".", "1e", "++0", "0_0", "0b2", "1.f", "\u{200b}"] {
            assert!(Number::from_string(invalid).is_nan(), "{invalid:?}");
        }
    }

    #[test]
    fn overflowing_radix_literals_convert_to_positive_infinity() {
        for (prefix, digit, count) in [("0b", '1', 1_100), ("0o", '7', 400), ("0x", 'f', 300)] {
            let literal = format!("{prefix}{}", digit.to_string().repeat(count));
            assert_eq!(Number::from_string(&literal), Number::infinity(1));
        }
    }

    #[test]
    fn handles_pseudo_bigints() {
        assert_eq!(parse_pseudo_big_int("00042n"), "42");
        assert_eq!(parse_pseudo_big_int("0b1010_0101n"), "165");
        assert_eq!(parse_pseudo_big_int("0o755n"), "493");
        assert_eq!(
            parse_pseudo_big_int("0x18ee90ff6c373e0ee4e3f0ad2n"),
            "123456789012345678901234567890"
        );
        let negative = PseudoBigInt::parse_valid("-00042n");
        assert_eq!(negative.to_string(), "-42");
        assert_eq!(negative.sign(), -1);
        assert_eq!(PseudoBigInt::new("000", true).sign(), 0);
    }
}
