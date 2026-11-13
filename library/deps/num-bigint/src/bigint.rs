// `num_bigint::BigInt`: a sign and a normalized magnitude (no high zero
// limbs; zero is `NoSign` with no limbs).

use core::cmp::Ordering;
use core::fmt;
use num_traits::ToPrimitive;

/// The sign of a `BigInt`, ordered Minus < NoSign < Plus.
#[derive(PartialEq, PartialOrd, Eq, Ord, Copy, Clone, Debug)]
pub enum Sign {
    Minus,
    NoSign,
    Plus,
}

/// An integer of any size.
#[derive(Clone)]
pub struct BigInt {
    sign: Sign,
    mag: Vec<u64>,
}

impl BigInt {
    /// Parses `buf` (UTF-8 text) as digits in `radix` (2 to 36) with an
    /// optional `-` or `+`; `_` may separate digits after the first one.
    /// None for an empty or invalid number. Panics on another radix, as
    /// upstream does, once the text is valid UTF-8.
    pub fn parse_bytes(buf: &[u8], radix: u32) -> Option<BigInt> {
        if !utf8_ok(buf) {
            return None;
        }
        assert!(2 <= radix && radix <= 36, "The radix must be within 2...36");
        let mut start = 0;
        let mut sign = Sign::Plus;
        if !buf.is_empty() && buf[0] == b'-' && !(buf.len() > 1 && buf[1] == b'+') {
            sign = Sign::Minus;
            start = 1;
        } else if !buf.is_empty() && buf[0] == b'-' {
            sign = Sign::Minus;
        }
        if start < buf.len() && buf[start] == b'+' && !(start + 1 < buf.len() && buf[start + 1] == b'+') {
            start += 1;
        }
        if start == buf.len() || buf[start] == b'_' {
            return None;
        }
        let mut mag: Vec<u64> = Vec::new();
        let mut i = start;
        while i < buf.len() {
            let b = buf[i];
            i += 1;
            let d = if b >= b'0' && b <= b'9' {
                (b - b'0') as u32
            } else if b >= b'a' && b <= b'z' {
                (b - b'a') as u32 + 10
            } else if b >= b'A' && b <= b'Z' {
                (b - b'A') as u32 + 10
            } else if b == b'_' {
                continue;
            } else {
                return None;
            };
            if d >= radix {
                return None;
            }
            mul_add_small(&mut mag, radix as u64, d as u64);
        }
        Some(from_parts(sign, mag))
    }

    /// The sign of this integer.
    pub fn sign(&self) -> Sign {
        self.sign
    }

    /// The number of significant bits of the magnitude (0 for zero).
    pub fn bits(&self) -> u64 {
        bits_of(&self.mag)
    }

    /// `self` raised to `exponent` (`0^0` is 1).
    pub fn pow(&self, exponent: u32) -> BigInt {
        let mut result: Vec<u64> = Vec::new();
        result.push(1);
        let mut base = self.mag.clone();
        let mut e = exponent;
        while e > 0 {
            if e & 1 == 1 {
                result = mul_mag(&result, &base);
            }
            e >>= 1;
            if e > 0 {
                base = mul_mag(&base, &base);
            }
        }
        let sign = if self.sign == Sign::Minus && exponent % 2 == 1 { Sign::Minus } else { Sign::Plus };
        from_parts(sign, result)
    }

    /// The digits in `radix` (2 to 36, lowercase), with `-` when negative.
    pub fn to_str_radix(&self, radix: u32) -> String {
        assert!(2 <= radix && radix <= 36, "The radix must be within 2...36");
        let mut out = String::new();
        if self.sign == Sign::Minus {
            out.push('-');
        }
        push_digits(&mut out, &self.mag, radix as u64);
        out
    }
}

/// Builds a normalized BigInt: zero always gets NoSign.
fn from_parts(sign: Sign, mut mag: Vec<u64>) -> BigInt {
    while !mag.is_empty() && mag[mag.len() - 1] == 0 {
        mag.pop();
    }
    if mag.is_empty() { BigInt { sign: Sign::NoSign, mag } } else { BigInt { sign, mag } }
}

/// True when `b` is valid UTF-8 (parse_bytes checks it before anything else).
fn utf8_ok(b: &[u8]) -> bool {
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        let (n, min) = if c < 0x80 {
            (0, 0u32)
        } else if c >= 0xC2 && c <= 0xDF {
            (1, 0x80u32)
        } else if c >= 0xE0 && c <= 0xEF {
            (2, 0x800u32)
        } else if c >= 0xF0 && c <= 0xF4 {
            (3, 0x10000u32)
        } else {
            return false;
        };
        if i + n >= b.len() {
            return false;
        }
        let mut cp = if n == 0 { c as u32 } else { (c as u32) & (0x3F >> n) };
        let mut k = 1;
        while k <= n {
            let cc = b[i + k];
            if cc & 0xC0 != 0x80 {
                return false;
            }
            cp = (cp << 6) | (cc as u32 & 0x3F);
            k += 1;
        }
        if cp < min || cp > 0x10FFFF || (cp >= 0xD800 && cp <= 0xDFFF) {
            return false;
        }
        i += n + 1;
    }
    true
}

/// `mag = mag * m + a` for small `m` and `a`.
fn mul_add_small(mag: &mut Vec<u64>, m: u64, a: u64) {
    let mut carry = a as u128;
    let mut i = 0;
    while i < mag.len() {
        let t = (mag[i] as u128) * (m as u128) + carry;
        mag[i] = t as u64;
        carry = t >> 64;
        i += 1;
    }
    if carry != 0 {
        mag.push(carry as u64);
    }
}

/// The schoolbook product of two magnitudes, normalized.
fn mul_mag(a: &[u64], b: &[u64]) -> Vec<u64> {
    let mut out: Vec<u64> = Vec::new();
    let mut k = 0;
    while k < a.len() + b.len() {
        out.push(0);
        k += 1;
    }
    let mut i = 0;
    while i < a.len() {
        let mut carry: u128 = 0;
        let mut j = 0;
        while j < b.len() {
            let t = (a[i] as u128) * (b[j] as u128) + (out[i + j] as u128) + carry;
            out[i + j] = t as u64;
            carry = t >> 64;
            j += 1;
        }
        let mut at = i + b.len();
        while carry != 0 {
            let t = (out[at] as u128) + carry;
            out[at] = t as u64;
            carry = t >> 64;
            at += 1;
        }
        i += 1;
    }
    while !out.is_empty() && out[out.len() - 1] == 0 {
        out.pop();
    }
    out
}

/// Divides `mag` in place by small `d`; returns the remainder.
fn div_small(mag: &mut Vec<u64>, d: u64) -> u64 {
    let mut rem: u128 = 0;
    let mut i = mag.len();
    while i > 0 {
        i -= 1;
        let cur = (rem << 64) | (mag[i] as u128);
        mag[i] = (cur / (d as u128)) as u64;
        rem = cur % (d as u128);
    }
    while !mag.is_empty() && mag[mag.len() - 1] == 0 {
        mag.pop();
    }
    rem as u64
}

/// Appends the magnitude's digits in `radix` ("0" for zero).
fn push_digits(out: &mut String, mag: &[u64], radix: u64) {
    if mag.is_empty() {
        out.push('0');
        return;
    }
    let mut rest = mag.to_vec();
    let mut digits: Vec<u8> = Vec::new();
    while !rest.is_empty() {
        let d = div_small(&mut rest, radix) as u8;
        digits.push(if d < 10 { b'0' + d } else { b'a' + d - 10 });
    }
    let mut i = digits.len();
    while i > 0 {
        i -= 1;
        out.push(digits[i] as char);
    }
}

fn bits_of(mag: &[u64]) -> u64 {
    if mag.is_empty() {
        return 0;
    }
    let top = mag[mag.len() - 1];
    (mag.len() as u64) * 64 - top.leading_zeros() as u64
}

/// The top 64 bits of the magnitude, with round-to-odd on the bits
/// below them, so one more rounding to f64 is correct (upstream's
/// high_bits_to_u64).
fn high_bits(mag: &[u64]) -> u64 {
    if mag.is_empty() {
        return 0;
    }
    if mag.len() == 1 {
        return mag[0];
    }
    let mut bits = bits_of(mag);
    let mut ret: u64 = 0;
    let mut ret_bits: u64 = 0;
    let mut i = mag.len();
    while i > 0 {
        i -= 1;
        let d = mag[i];
        let digit_bits = (bits - 1) % 64 + 1;
        let bits_want = if 64 - ret_bits < digit_bits { 64 - ret_bits } else { digit_bits };
        if bits_want != 0 {
            if bits_want != 64 {
                ret <<= bits_want;
            }
            ret |= d >> (digit_bits - bits_want);
        }
        if digit_bits - bits_want != 0 {
            let masked = d << (64 - (digit_bits - bits_want) as u32);
            if masked != 0 {
                ret |= 1;
            }
        }
        ret_bits += bits_want;
        bits -= bits_want;
    }
    ret
}

/// The magnitude as f64: upstream's (high_bits as f64) * 2^exponent.
fn mag_to_f64(mag: &[u64]) -> f64 {
    let mantissa = high_bits(mag);
    let fls = 64 - mantissa.leading_zeros() as u64;
    let exponent = bits_of(mag) - fls;
    if exponent > 1024 {
        return f64::INFINITY;
    }
    let scale = if exponent == 1024 { f64::INFINITY } else { f64::from_bits((1023 + exponent) << 52) };
    (mantissa as f64) * scale
}

impl ToPrimitive for BigInt {
    fn to_i64(&self) -> Option<i64> {
        let n = self.to_magnitude_u64()?;
        match self.sign {
            Sign::NoSign => Some(0),
            Sign::Plus => {
                if n <= 9223372036854775807 { Some(n as i64) } else { None }
            }
            Sign::Minus => {
                if n < 9223372036854775808 {
                    Some(-(n as i64))
                } else if n == 9223372036854775808 {
                    Some(-9223372036854775807 - 1)
                } else {
                    None
                }
            }
        }
    }

    fn to_u64(&self) -> Option<u64> {
        match self.sign {
            Sign::NoSign => Some(0),
            Sign::Plus => self.to_magnitude_u64(),
            Sign::Minus => None,
        }
    }

    fn to_i128(&self) -> Option<i128> {
        let n = self.to_magnitude_u128()?;
        let top: u128 = 1 << 127;
        match self.sign {
            Sign::NoSign => Some(0),
            Sign::Plus => {
                if n < top { Some(n as i128) } else { None }
            }
            Sign::Minus => {
                if n < top {
                    Some(-(n as i128))
                } else if n == top {
                    Some(-170141183460469231731687303715884105727 - 1)
                } else {
                    None
                }
            }
        }
    }

    fn to_u128(&self) -> Option<u128> {
        match self.sign {
            Sign::NoSign => Some(0),
            Sign::Plus => self.to_magnitude_u128(),
            Sign::Minus => None,
        }
    }

    fn to_f64(&self) -> Option<f64> {
        let n = mag_to_f64(&self.mag);
        Some(if self.sign == Sign::Minus { -n } else { n })
    }
}

impl BigInt {
    fn to_magnitude_u64(&self) -> Option<u64> {
        match self.mag.len() {
            0 => Some(0),
            1 => Some(self.mag[0]),
            _ => None,
        }
    }

    fn to_magnitude_u128(&self) -> Option<u128> {
        match self.mag.len() {
            0 => Some(0),
            1 => Some(self.mag[0] as u128),
            2 => Some((self.mag[0] as u128) | ((self.mag[1] as u128) << 64)),
            _ => None,
        }
    }
}

impl From<i64> for BigInt {
    fn from(n: i64) -> BigInt {
        if n < 0 {
            let mut mag: Vec<u64> = Vec::new();
            mag.push((-(n as i128)) as u64);
            from_parts(Sign::Minus, mag)
        } else {
            let mut mag: Vec<u64> = Vec::new();
            mag.push(n as u64);
            from_parts(Sign::Plus, mag)
        }
    }
}

impl From<u64> for BigInt {
    fn from(n: u64) -> BigInt {
        let mut mag: Vec<u64> = Vec::new();
        mag.push(n);
        from_parts(Sign::Plus, mag)
    }
}

impl From<i32> for BigInt {
    fn from(n: i32) -> BigInt {
        BigInt::from(n as i64)
    }
}

impl From<u32> for BigInt {
    fn from(n: u32) -> BigInt {
        BigInt::from(n as u64)
    }
}

impl Default for BigInt {
    fn default() -> BigInt {
        BigInt { sign: Sign::NoSign, mag: Vec::new() }
    }
}

impl PartialEq for BigInt {
    fn eq(&self, other: &BigInt) -> bool {
        self.sign == other.sign && self.mag == other.mag
    }
}

impl Eq for BigInt {}

impl PartialOrd for BigInt {
    fn partial_cmp(&self, other: &BigInt) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BigInt {
    fn cmp(&self, other: &BigInt) -> Ordering {
        let scmp = self.sign.cmp(&other.sign);
        if scmp != Ordering::Equal {
            return scmp;
        }
        match self.sign {
            Sign::NoSign => Ordering::Equal,
            Sign::Plus => cmp_mag(&self.mag, &other.mag),
            Sign::Minus => cmp_mag(&other.mag, &self.mag),
        }
    }
}

fn cmp_mag(a: &[u64], b: &[u64]) -> Ordering {
    if a.len() != b.len() {
        return a.len().cmp(&b.len());
    }
    let mut i = a.len();
    while i > 0 {
        i -= 1;
        if a[i] != b[i] {
            return a[i].cmp(&b[i]);
        }
    }
    Ordering::Equal
}

/// Decimal digits through `pad_integral`, so width, fill, `+`, and `0`
/// apply as for primitive integers.
impl fmt::Display for BigInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        pad_decimal(self, f)
    }
}

/// Debug prints as Display, as upstream does.
impl fmt::Debug for BigInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        pad_decimal(self, f)
    }
}

fn pad_decimal(b: &BigInt, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut digits = String::new();
    push_digits(&mut digits, &b.mag, 10);
    f.pad_integral(b.sign != Sign::Minus, "", &digits)
}
