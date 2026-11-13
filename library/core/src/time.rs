// `core::time` (docs/ARCH.md section 3.1; W-STD-C): `Duration` and
// `TryFromFloatSecsError`.
//
// Lust std source, adapted from rust-lang/rust 1.99 `core/src/time.rs`.
// The std loader splices this file into a user compile as one hidden
// module (`STD_MODULES` in src/stdmods.rs); `core::time::*` paths walk
// into it, and `std::time` re-exports its items. It is never fed to
// rustc as a replacement std; it only has to behave the same through
// the public API.
//
// Rules for std source modules (rule zero):
// - Only stable items. An unstable item here would let a user program
//   call it where rustc rejects (`Duration::SECOND`, `from_days`, ...).
// - `const fn` exactly where std has `const fn`, so const contexts
//   accept and reject what rustc does.
// - Std's niche field type `Nanoseconds` is a plain `u32` here. Layout
//   queries on std module types reject, so the missing niche cannot
//   show.
// - Panic messages are the std ones.
// - Associated const values name their own type as `Self`.

const NANOS_PER_SEC: u32 = 1_000_000_000;
const NANOS_PER_MILLI: u32 = 1_000_000;
const NANOS_PER_MICRO: u32 = 1_000;
const MILLIS_PER_SEC: u64 = 1_000;
const MICROS_PER_SEC: u64 = 1_000_000;
const SECS_PER_MINUTE: u64 = 60;
const MINS_PER_HOUR: u64 = 60;

/// A span of time: whole seconds plus nanoseconds (always below one
/// second).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Duration {
    secs: u64,
    nanos: u32,
}

impl Duration {
    pub const ZERO: Duration = Self::from_nanos(0);
    pub const MAX: Duration = Self::new(u64::MAX, NANOS_PER_SEC - 1);

    pub const fn new(secs: u64, nanos: u32) -> Duration {
        if nanos < NANOS_PER_SEC {
            Duration { secs, nanos }
        } else {
            let secs = match secs.checked_add((nanos / NANOS_PER_SEC) as u64) {
                Some(s) => s,
                None => panic!("overflow in Duration::new"), //@ core/src/time.rs:201:18
            };
            let nanos = nanos % NANOS_PER_SEC;
            Duration { secs, nanos }
        }
    }

    pub const fn from_secs(secs: u64) -> Duration {
        Duration { secs, nanos: 0 }
    }

    pub const fn from_millis(millis: u64) -> Duration {
        let secs = millis / MILLIS_PER_SEC;
        let subsec_millis = (millis % MILLIS_PER_SEC) as u32;
        Duration { secs, nanos: subsec_millis * NANOS_PER_MILLI }
    }

    pub const fn from_micros(micros: u64) -> Duration {
        let secs = micros / MICROS_PER_SEC;
        let subsec_micros = (micros % MICROS_PER_SEC) as u32;
        Duration { secs, nanos: subsec_micros * NANOS_PER_MICRO }
    }

    pub const fn from_nanos(nanos: u64) -> Duration {
        let secs = nanos / (NANOS_PER_SEC as u64);
        let subsec_nanos = (nanos % (NANOS_PER_SEC as u64)) as u32;
        Duration { secs, nanos: subsec_nanos }
    }

    #[track_caller]
    pub const fn from_nanos_u128(nanos: u128) -> Duration {
        let secs = nanos / (NANOS_PER_SEC as u128);
        if secs > u64::MAX as u128 {
            panic!("overflow in `Duration::from_nanos_u128`");
        }
        let subsec_nanos = (nanos % (NANOS_PER_SEC as u128)) as u32;
        Duration { secs: secs as u64, nanos: subsec_nanos }
    }

    pub const fn from_hours(hours: u64) -> Duration {
        if hours > u64::MAX / (SECS_PER_MINUTE * MINS_PER_HOUR) {
            panic!("overflow in Duration::from_hours"); //@ core/src/time.rs:424:13
        }
        Duration::from_secs(hours * MINS_PER_HOUR * SECS_PER_MINUTE)
    }

    pub const fn from_mins(mins: u64) -> Duration {
        if mins > u64::MAX / SECS_PER_MINUTE {
            panic!("overflow in Duration::from_mins"); //@ core/src/time.rs:452:13
        }
        Duration::from_secs(mins * SECS_PER_MINUTE)
    }

    pub const fn is_zero(&self) -> bool {
        self.secs == 0 && self.nanos == 0
    }

    pub const fn as_secs(&self) -> u64 {
        self.secs
    }

    pub const fn subsec_millis(&self) -> u32 {
        self.nanos / NANOS_PER_MILLI
    }

    pub const fn subsec_micros(&self) -> u32 {
        self.nanos / NANOS_PER_MICRO
    }

    pub const fn subsec_nanos(&self) -> u32 {
        self.nanos
    }

    pub const fn as_millis(&self) -> u128 {
        self.secs as u128 * MILLIS_PER_SEC as u128 + (self.nanos / NANOS_PER_MILLI) as u128
    }

    pub const fn as_micros(&self) -> u128 {
        self.secs as u128 * MICROS_PER_SEC as u128 + (self.nanos / NANOS_PER_MICRO) as u128
    }

    pub const fn as_nanos(&self) -> u128 {
        self.secs as u128 * NANOS_PER_SEC as u128 + self.nanos as u128
    }

    pub const fn abs_diff(self, other: Duration) -> Duration {
        if let Some(res) = self.checked_sub(other) { res } else { other.checked_sub(self).unwrap() } //@ never
    }

    pub const fn checked_add(self, rhs: Duration) -> Option<Duration> {
        if let Some(mut secs) = self.secs.checked_add(rhs.secs) {
            let mut nanos = self.nanos + rhs.nanos;
            if nanos >= NANOS_PER_SEC {
                nanos -= NANOS_PER_SEC;
                if let Some(new_secs) = secs.checked_add(1) {
                    secs = new_secs;
                } else {
                    return None;
                }
            }
            Some(Duration::new(secs, nanos))
        } else {
            None
        }
    }

    pub const fn saturating_add(self, rhs: Duration) -> Duration {
        match self.checked_add(rhs) {
            Some(res) => res,
            None => Duration::MAX,
        }
    }

    pub const fn checked_sub(self, rhs: Duration) -> Option<Duration> {
        if let Some(mut secs) = self.secs.checked_sub(rhs.secs) {
            let nanos = if self.nanos >= rhs.nanos {
                self.nanos - rhs.nanos
            } else if let Some(sub_secs) = secs.checked_sub(1) {
                secs = sub_secs;
                self.nanos + NANOS_PER_SEC - rhs.nanos
            } else {
                return None;
            };
            Some(Duration::new(secs, nanos))
        } else {
            None
        }
    }

    pub const fn saturating_sub(self, rhs: Duration) -> Duration {
        match self.checked_sub(rhs) {
            Some(res) => res,
            None => Duration::ZERO,
        }
    }

    pub const fn checked_mul(self, rhs: u32) -> Option<Duration> {
        let total_nanos = self.nanos as u64 * rhs as u64;
        let extra_secs = total_nanos / (NANOS_PER_SEC as u64);
        let nanos = (total_nanos % (NANOS_PER_SEC as u64)) as u32;
        if let Some(s) = self.secs.checked_mul(rhs as u64) {
            if let Some(secs) = s.checked_add(extra_secs) {
                return Some(Duration::new(secs, nanos));
            }
        }
        None
    }

    pub const fn saturating_mul(self, rhs: u32) -> Duration {
        match self.checked_mul(rhs) {
            Some(res) => res,
            None => Duration::MAX,
        }
    }

    pub const fn checked_div(self, rhs: u32) -> Option<Duration> {
        if rhs != 0 {
            let secs = self.secs / (rhs as u64);
            let extra_secs = self.secs % (rhs as u64);
            let mut nanos = self.nanos / rhs;
            let extra_nanos = self.nanos % rhs;
            nanos += ((extra_secs * (NANOS_PER_SEC as u64) + extra_nanos as u64) / (rhs as u64)) as u32;
            Some(Duration::new(secs, nanos))
        } else {
            None
        }
    }

    pub const fn as_secs_f64(&self) -> f64 {
        (self.secs as f64) + (self.nanos as f64) / (NANOS_PER_SEC as f64)
    }

    pub const fn as_secs_f32(&self) -> f32 {
        (self.secs as f32) + (self.nanos as f32) / (NANOS_PER_SEC as f32)
    }

    pub fn from_secs_f64(secs: f64) -> Duration {
        match Duration::try_from_secs_f64(secs) {
            Ok(v) => v,
            Err(e) => panic!("{}", e), //@ core/src/time.rs:962:23
        }
    }

    pub fn from_secs_f32(secs: f32) -> Duration {
        match Duration::try_from_secs_f32(secs) {
            Ok(v) => v,
            Err(e) => panic!("{}", e), //@ core/src/time.rs:999:23
        }
    }

    pub fn mul_f64(self, rhs: f64) -> Duration {
        Duration::from_secs_f64(rhs * self.as_secs_f64())
    }

    pub fn mul_f32(self, rhs: f32) -> Duration {
        self.mul_f64(rhs as f64)
    }

    pub fn div_f64(self, rhs: f64) -> Duration {
        Duration::from_secs_f64(self.as_secs_f64() / rhs)
    }

    pub fn div_f32(self, rhs: f32) -> Duration {
        self.div_f64(rhs as f64)
    }

    pub const fn div_duration_f64(self, rhs: Duration) -> f64 {
        let self_nanos = (self.secs as f64) * (NANOS_PER_SEC as f64) + (self.nanos as f64);
        let rhs_nanos = (rhs.secs as f64) * (NANOS_PER_SEC as f64) + (rhs.nanos as f64);
        self_nanos / rhs_nanos
    }

    pub const fn div_duration_f32(self, rhs: Duration) -> f32 {
        let self_nanos = (self.secs as f32) * (NANOS_PER_SEC as f32) + (self.nanos as f32);
        let rhs_nanos = (rhs.secs as f32) * (NANOS_PER_SEC as f32) + (rhs.nanos as f32);
        self_nanos / rhs_nanos
    }

    /// `try_from_secs!` of core with f32 parameters (mantissa 23 bits,
    /// exponent 8 bits, offset 41, double type u64).
    pub fn try_from_secs_f32(secs: f32) -> Result<Duration, TryFromFloatSecsError> {
        if secs < 0.0 {
            return Err(TryFromFloatSecsError { kind: TryFromFloatSecsErrorKind::Negative });
        }
        let bits: u32 = secs.to_bits();
        let mant: u32 = (bits & 0x7f_ffff) | 0x80_0000;
        let exp: i16 = ((bits >> 23) & 0xff) as i16 - 127;
        let mut secs_out: u64 = 0;
        let mut nanos_out: u32 = 0;
        if exp < -31 {
            // Less than 1ns: rounds to zero.
        } else if exp < 0 {
            let t: u64 = (mant as u64) << ((41 + exp) as u32);
            let nanos_offset: u32 = 23 + 41;
            let nanos_tmp: u128 = NANOS_PER_SEC as u128 * t as u128;
            let nanos = (nanos_tmp >> nanos_offset) as u32;
            let rem_mask: u128 = (1u128 << nanos_offset) - 1;
            let rem_msb_mask: u128 = 1u128 << (nanos_offset - 1);
            let rem = nanos_tmp & rem_mask;
            let is_tie = rem == rem_msb_mask;
            let is_even = (nanos & 1) == 0;
            let rem_msb = nanos_tmp & rem_msb_mask == 0;
            let add_ns = !(rem_msb || (is_even && is_tie));
            nanos_out = nanos + add_ns as u32;
        } else if exp < 23 {
            secs_out = (mant >> ((23 - exp) as u32)) as u64;
            let t: u64 = ((mant << (exp as u32)) & 0x7f_ffff) as u64;
            let nanos_offset: u32 = 23;
            let nanos_tmp: u64 = NANOS_PER_SEC as u64 * t;
            let nanos = (nanos_tmp >> nanos_offset) as u32;
            let rem_mask: u64 = (1u64 << nanos_offset) - 1;
            let rem_msb_mask: u64 = 1u64 << (nanos_offset - 1);
            let rem = nanos_tmp & rem_mask;
            let is_tie = rem == rem_msb_mask;
            let is_even = (nanos & 1) == 0;
            let rem_msb = nanos_tmp & rem_msb_mask == 0;
            let add_ns = !(rem_msb || (is_even && is_tie));
            nanos_out = nanos + add_ns as u32;
        } else if exp < 64 {
            secs_out = (mant as u64) << ((exp - 23) as u32);
        } else {
            return Err(TryFromFloatSecsError { kind: TryFromFloatSecsErrorKind::OverflowOrNan });
        }
        Ok(Duration::new(secs_out, nanos_out))
    }

    /// `try_from_secs!` of core with f64 parameters (mantissa 52 bits,
    /// exponent 11 bits, offset 44, double type u128).
    pub fn try_from_secs_f64(secs: f64) -> Result<Duration, TryFromFloatSecsError> {
        if secs < 0.0 {
            return Err(TryFromFloatSecsError { kind: TryFromFloatSecsErrorKind::Negative });
        }
        let bits: u64 = secs.to_bits();
        let mant: u64 = (bits & 0xf_ffff_ffff_ffff) | 0x10_0000_0000_0000;
        let exp: i16 = ((bits >> 52) & 0x7ff) as i16 - 1023;
        let mut secs_out: u64 = 0;
        let mut nanos_out: u32 = 0;
        if exp < -31 {
            // Less than 1ns: rounds to zero.
        } else if exp < 0 {
            let t: u128 = (mant as u128) << ((44 + exp) as u32);
            let nanos_offset: u32 = 52 + 44;
            let nanos_tmp: u128 = NANOS_PER_SEC as u128 * t;
            let nanos = (nanos_tmp >> nanos_offset) as u32;
            let rem_mask: u128 = (1u128 << nanos_offset) - 1;
            let rem_msb_mask: u128 = 1u128 << (nanos_offset - 1);
            let rem = nanos_tmp & rem_mask;
            let is_tie = rem == rem_msb_mask;
            let is_even = (nanos & 1) == 0;
            let rem_msb = nanos_tmp & rem_msb_mask == 0;
            let add_ns = !(rem_msb || (is_even && is_tie));
            let nanos = nanos + add_ns as u32;
            if nanos != NANOS_PER_SEC {
                nanos_out = nanos;
            } else {
                secs_out = 1;
            }
        } else if exp < 52 {
            secs_out = mant >> ((52 - exp) as u32);
            let t: u128 = ((mant << (exp as u32)) & 0xf_ffff_ffff_ffff) as u128;
            let nanos_offset: u32 = 52;
            let nanos_tmp: u128 = NANOS_PER_SEC as u128 * t;
            let nanos = (nanos_tmp >> nanos_offset) as u32;
            let rem_mask: u128 = (1u128 << nanos_offset) - 1;
            let rem_msb_mask: u128 = 1u128 << (nanos_offset - 1);
            let rem = nanos_tmp & rem_mask;
            let is_tie = rem == rem_msb_mask;
            let is_even = (nanos & 1) == 0;
            let rem_msb = nanos_tmp & rem_msb_mask == 0;
            let add_ns = !(rem_msb || (is_even && is_tie));
            let nanos = nanos + add_ns as u32;
            if nanos != NANOS_PER_SEC {
                nanos_out = nanos;
            } else {
                secs_out += 1;
            }
        } else if exp < 64 {
            secs_out = mant << ((exp - 52) as u32);
        } else {
            return Err(TryFromFloatSecsError { kind: TryFromFloatSecsErrorKind::OverflowOrNan });
        }
        Ok(Duration::new(secs_out, nanos_out))
    }
}

impl core::ops::Add for Duration {
    type Output = Duration;
    fn add(self, rhs: Duration) -> Duration {
        self.checked_add(rhs).expect("overflow when adding durations") //@ core/src/time.rs:1257:31
    }
}

impl core::ops::AddAssign for Duration {
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs;
    }
}

impl core::ops::Sub for Duration {
    type Output = Duration;
    fn sub(self, rhs: Duration) -> Duration {
        self.checked_sub(rhs).expect("overflow when subtracting durations") //@ core/src/time.rs:1277:31
    }
}

impl core::ops::SubAssign for Duration {
    fn sub_assign(&mut self, rhs: Duration) {
        *self = *self - rhs;
    }
}

impl core::ops::Mul<u32> for Duration {
    type Output = Duration;
    fn mul(self, rhs: u32) -> Duration {
        self.checked_mul(rhs).expect("overflow when multiplying duration by scalar") //@ core/src/time.rs:1297:31
    }
}

impl core::ops::Mul<Duration> for u32 {
    type Output = Duration;
    fn mul(self, rhs: Duration) -> Duration {
        rhs * self
    }
}

impl core::ops::MulAssign<u32> for Duration {
    fn mul_assign(&mut self, rhs: u32) {
        *self = *self * rhs;
    }
}

impl core::ops::Div<u32> for Duration {
    type Output = Duration;
    #[track_caller]
    fn div(self, rhs: u32) -> Duration {
        self.checked_div(rhs).expect("divide by zero error when dividing duration by scalar")
    }
}

impl core::ops::DivAssign<u32> for Duration {
    #[track_caller]
    fn div_assign(&mut self, rhs: u32) {
        *self = *self / rhs;
    }
}

/// The decimal digit count of `n` (1 for 0).
fn dec_len(n: u64) -> usize {
    let mut len = 1;
    let mut n = n;
    while n >= 10 {
        n /= 10;
        len += 1;
    }
    len
}

/// Writes `n` copies of the formatter's fill char.
fn write_fill(f: &mut core::fmt::Formatter<'_>, n: usize) -> core::fmt::Result {
    let fill = f.fill();
    let mut i = 0;
    while i < n {
        write!(f, "{}", fill)?;
        i += 1;
    }
    Ok(())
}

/// Core's `fmt_decimal` for `Duration`'s Debug: `integer_part` plus the
/// fraction `fractional_part / divisor`, rounded to the precision
/// (ties to even), with a prefix and a unit postfix, padded to the
/// width (left aligned unless the format says otherwise).
fn fmt_decimal(
    f: &mut core::fmt::Formatter<'_>,
    integer_part: u64,
    fractional_part: u32,
    divisor: u32,
    prefix: &str,
    postfix: &str,
) -> core::fmt::Result {
    let mut fractional_part = fractional_part;
    let mut divisor = divisor;
    let mut buf = [b'0'; 9];
    let mut pos: usize = 0;
    let max_digits = match f.precision() {
        Some(p) => p,
        None => 9,
    };
    while fractional_part > 0 && pos < max_digits {
        buf[pos] = b'0' + (fractional_part / divisor) as u8;
        fractional_part %= divisor;
        divisor /= 10;
        pos += 1;
    }
    // `None` stands for u64::MAX + 1 after a rounding carry.
    let mut integer: Option<u64> = Some(integer_part);
    if fractional_part > 0 && fractional_part >= divisor * 5 {
        let is_tie = fractional_part == divisor * 5;
        let last_digit_is_odd = if pos > 0 { (buf[pos - 1] - b'0') % 2 == 1 } else { (integer_part % 2) == 1 };
        if !(is_tie && !last_digit_is_odd) {
            let mut rev_pos = pos;
            let mut carry = true;
            while carry && rev_pos > 0 {
                rev_pos -= 1;
                if buf[rev_pos] < b'9' {
                    buf[rev_pos] += 1;
                    carry = false;
                } else {
                    buf[rev_pos] = b'0';
                }
            }
            if carry {
                integer = integer_part.checked_add(1);
            }
        }
    }
    let end = match f.precision() {
        Some(p) => {
            if p < 9 {
                p
            } else {
                9
            }
        }
        None => pos,
    };
    let frac_w = match f.precision() {
        Some(p) => p,
        None => pos,
    };
    let mut actual_w = prefix.len() + postfix.chars().count();
    actual_w += match integer {
        Some(n) => dec_len(n),
        None => 20,
    };
    if end > 0 {
        actual_w += 1 + frac_w;
    }
    let pad = match f.width() {
        Some(w) => {
            if w > actual_w {
                w - actual_w
            } else {
                0
            }
        }
        None => 0,
    };
    let (pre, post) = match core::intrinsics::fmt_align(f) {
        1 => (pad, 0),
        2 => (pad / 2, (pad + 1) / 2),
        _ => (0, pad),
    };
    write_fill(f, pre)?;
    match integer {
        Some(n) => write!(f, "{}{}", prefix, n)?,
        None => write!(f, "{}18446744073709551616", prefix)?,
    }
    if end > 0 {
        f.write_str(".")?;
        let mut i = 0;
        while i < end {
            write!(f, "{}", buf[i] as char)?;
            i += 1;
        }
        while i < frac_w {
            f.write_str("0")?;
            i += 1;
        }
    }
    f.write_str(postfix)?;
    write_fill(f, post)
}

impl core::fmt::Debug for Duration {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let prefix = if f.sign_plus() { "+" } else { "" };
        if self.secs > 0 {
            fmt_decimal(f, self.secs, self.nanos, NANOS_PER_SEC / 10, prefix, "s")
        } else if self.nanos >= NANOS_PER_MILLI {
            fmt_decimal(f, (self.nanos / NANOS_PER_MILLI) as u64, self.nanos % NANOS_PER_MILLI, NANOS_PER_MILLI / 10, prefix, "ms")
        } else if self.nanos >= NANOS_PER_MICRO {
            fmt_decimal(f, (self.nanos / NANOS_PER_MICRO) as u64, self.nanos % NANOS_PER_MICRO, NANOS_PER_MICRO / 10, prefix, "µs")
        } else {
            fmt_decimal(f, self.nanos as u64, 0, 1, prefix, "ns")
        }
    }
}

/// The error of `Duration::try_from_secs_f32` and `try_from_secs_f64`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TryFromFloatSecsError {
    kind: TryFromFloatSecsErrorKind,
}

impl core::fmt::Display for TryFromFloatSecsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let msg = match self.kind {
            TryFromFloatSecsErrorKind::Negative => "cannot convert float seconds to Duration: value is negative",
            TryFromFloatSecsErrorKind::OverflowOrNan => "cannot convert float seconds to Duration: value is either too big or NaN",
        };
        f.pad(msg)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TryFromFloatSecsErrorKind {
    Negative,
    OverflowOrNan,
}

