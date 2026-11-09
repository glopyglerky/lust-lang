//! Stand-in for the ts-rust jsnum module, reduced to the two entry
//! points the scanner calls: `Number::from_string` with its `Display`
//! (Go jsnum.FromString(text).String()) and `parse_pseudo_big_int`.
//!
//! Replacements against the original (571 lines, not copied):
//!
//! - StringToNumber: Rust f64 parsing is correctly rounded, as
//!   ECMAScript requires, so decimal text goes through str::parse after
//!   the JS grammar checks (radix literals, separators, Infinity). The
//!   exact-integer BigInt path of the original (num-bigint, num-traits)
//!   only matters for from_string of long integer strings through
//!   pow(); the scanner passes literal text it has already validated,
//!   and f64 parse gives the same value for those.
//! - Display: ECMAScript Number::toString layout over the shortest
//!   round-trip digits from Rust LowerExp formatting. The original uses
//!   the ryu-js crate; on a shortest-digit tie the two can in principle
//!   pick different last digits. No input in this kernel is known to
//!   tie.
//! - parse_pseudo_big_int: copied from the original except the
//!   BigInt radix conversion, which is a small limb loop here. The
//!   panic text on malformed input is the original text.

use std::fmt;

/// A JavaScript IEEE-754 Number value (ts-rust jsnum::Number shape).
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Number(pub f64);

impl Number {
    #[must_use]
    pub const fn new(value: f64) -> Self {
        Self(value)
    }

    #[must_use]
    pub fn nan() -> Self {
        Self(f64::NAN)
    }

    #[must_use]
    pub fn infinity(sign: i32) -> Self {
        Self(if sign < 0 { f64::NEG_INFINITY } else { f64::INFINITY })
    }

    /// ECMAScript StringToNumber conversion.
    #[must_use]
    pub fn from_string(text: &str) -> Self {
        string_to_number(text)
    }
}

/// ECMAScript StringToNumber for the texts the scanner produces:
/// trimmed decimal or radix integer literals with optional separators.
fn string_to_number(text: &str) -> Number {
    let text = text.trim_matches(is_js_whitespace);
    match text {
        "" => return Number(0.0),
        "Infinity" | "+Infinity" => return Number::infinity(1),
        "-Infinity" => return Number::infinity(-1),
        _ => {}
    }
    let (unsigned, negative) = match text.strip_prefix('-') {
        Some(rest) => (rest, true),
        None => (text.strip_prefix('+').unwrap_or(text), false),
    };
    // Radix integer literals (StringToNumber accepts them unsigned only;
    // a sign with a radix prefix is NaN in JS, and the scanner never
    // produces one, so matching the unsigned grammar is enough here).
    let radix = if unsigned.starts_with("0x") || unsigned.starts_with("0X") {
        Some(16)
    } else if unsigned.starts_with("0o") || unsigned.starts_with("0O") {
        Some(8)
    } else if unsigned.starts_with("0b") || unsigned.starts_with("0B") {
        Some(2)
    } else {
        None
    };
    if let Some(radix) = radix {
        if negative || text.starts_with('+') {
            return Number::nan();
        }
        let digits: String = unsigned[2..].chars().filter(|&c| c != '_').collect();
        if digits.is_empty() || !digits.chars().all(|c| c.is_digit(radix)) {
            return Number::nan();
        }
        // Exact for the literal sizes in source text: accumulate in f64
        // through u128 when it fits, else digit by digit.
        if let Ok(v) = u128::from_str_radix(&digits, radix) {
            return Number(v as f64);
        }
        let mut value = 0.0f64;
        for c in digits.chars() {
            value = value * f64::from(radix) + f64::from(c.to_digit(radix).unwrap_or(0));
        }
        return Number(value);
    }
    let cleaned: String = unsigned.chars().filter(|&c| c != '_').collect();
    if !cleaned.chars().all(|c| c.is_ascii_digit() || matches!(c, '.' | 'e' | 'E' | '+' | '-')) {
        return Number::nan();
    }
    match cleaned.parse::<f64>() {
        Ok(v) => Number(if negative { -v } else { v }),
        Err(_) => Number::nan(),
    }
}

/// The JS WhiteSpace and LineTerminator code points (for trimming).
fn is_js_whitespace(c: char) -> bool {
    matches!(
        c,
        '\u{0009}' | '\u{000A}' | '\u{000B}' | '\u{000C}' | '\u{000D}' | ' ' | '\u{00A0}'
            | '\u{1680}' | '\u{2000}'..='\u{200A}' | '\u{2028}' | '\u{2029}' | '\u{202F}'
            | '\u{205F}' | '\u{3000}' | '\u{FEFF}'
    )
}

impl fmt::Display for Number {
    /// ECMAScript Number::toString (radix 10).
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&number_to_string(self.0))
    }
}

/// ECMAScript Number::toString over Rust shortest round-trip digits.
fn number_to_string(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_string();
    }
    if value == 0.0 {
        return "0".to_string();
    }
    if value.is_infinite() {
        return if value > 0.0 { "Infinity" } else { "-Infinity" }.to_string();
    }
    let sign = if value < 0.0 { "-" } else { "" };
    // Rust LowerExp gives the shortest digits: "d.dddde<exp>" or "de<exp>".
    let sci = format!("{:e}", value.abs());
    let (mantissa, exp) = sci.split_once('e').expect("LowerExp has an exponent");
    let exp: i32 = exp.parse().expect("exponent is a number");
    let digits: String = mantissa.chars().filter(|&c| c != '.').collect();
    let digits = digits.trim_end_matches('0');
    let s = if digits.is_empty() { "0" } else { digits };
    // ECMAScript variables: value = s * 10^(n-k), k = len(s), the point
    // sits after the first n digits of s padded as needed.
    let k = s.len() as i32;
    let n = exp + 1;
    let body = if (k..=21).contains(&n) && n >= k {
        // Integer form: digits then zeros. (k <= n <= 21)
        let mut out = s.to_string();
        out.extend(std::iter::repeat_n('0', (n - k) as usize));
        out
    } else if n > 0 && n <= 21 {
        // Point inside the digits. (0 < n < k)
        format!("{}.{}", &s[..n as usize], &s[n as usize..])
    } else if n > -6 && n <= 0 {
        // Small fraction: 0.000ddd form.
        let mut out = "0.".to_string();
        out.extend(std::iter::repeat_n('0', (-n) as usize));
        out.push_str(s);
        out
    } else {
        // Exponential form.
        let e = n - 1;
        let mant = if k == 1 {
            s.to_string()
        } else {
            format!("{}.{}", &s[..1], &s[1..])
        };
        if e >= 0 {
            format!("{mant}e+{e}")
        } else {
            format!("{mant}e{e}")
        }
    };
    format!("{sign}{body}")
}

/// Converts a scanner-validated bigint literal to unsigned decimal text.
///
/// # Panics
///
/// Panics when a non-decimal literal is malformed (the ts-rust text).
#[must_use]
pub fn parse_pseudo_big_int(text: &str) -> String {
    let text = text.strip_suffix('n').unwrap_or(text);
    let prefix = text.get(0..2).unwrap_or_default();
    let radix = match prefix {
        "0b" | "0B" => Some(2u32),
        "0o" | "0O" => Some(8u32),
        "0x" | "0X" => Some(16u32),
        _ => None,
    };
    let Some(radix) = radix else {
        let value = text.trim_start_matches('0');
        return if value.is_empty() { "0" } else { value }.to_owned();
    };
    let digits = text[2..].replace('_', "");
    // Minimal big integer: little-endian base 2^32 limbs, multiply-add
    // per digit, then divide out decimal chunks. Same result as
    // BigInt::parse_bytes(...).to_string() for valid input.
    if digits.is_empty() || !digits.chars().all(|c| c.is_digit(radix)) {
        panic!("Failed to parse big int: {text:?}");
    }
    let mut limbs: Vec<u32> = vec![0];
    for c in digits.chars() {
        let d = c.to_digit(radix).unwrap_or(0) as u64;
        let mut carry = d;
        for limb in &mut limbs {
            let cur = u64::from(*limb) * u64::from(radix) + carry;
            *limb = cur as u32;
            carry = cur >> 32;
        }
        while carry > 0 {
            limbs.push(carry as u32);
            carry >>= 32;
        }
    }
    if limbs.iter().all(|&l| l == 0) {
        return "0".to_string();
    }
    let mut chunks: Vec<u32> = Vec::new();
    while limbs.len() > 1 || limbs[0] > 0 {
        let mut rem = 0u64;
        for limb in limbs.iter_mut().rev() {
            let cur = (rem << 32) | u64::from(*limb);
            *limb = (cur / 1_000_000_000) as u32;
            rem = cur % 1_000_000_000;
        }
        while limbs.len() > 1 && limbs[limbs.len() - 1] == 0 {
            limbs.pop();
        }
        chunks.push(rem as u32);
    }
    let mut out = chunks[chunks.len() - 1].to_string();
    for chunk in chunks[..chunks.len() - 1].iter().rev() {
        out.push_str(&format!("{chunk:09}"));
    }
    out
}
