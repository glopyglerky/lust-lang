// `num_traits::cast::ToPrimitive`: checked conversions to primitive
// numbers. The provided methods give upstream's answers: a conversion
// is Some exactly when the value is in the target's range. `to_f32` is
// left out (it rejects at the call): the u64-to-f32 rounding it needs is
// not proven exact under lust's C backend yet.

/// Checked conversions to the primitive numbers.
pub trait ToPrimitive {
    fn to_i64(&self) -> Option<i64>;

    fn to_u64(&self) -> Option<u64>;

    fn to_isize(&self) -> Option<isize> {
        let v = self.to_i64()?;
        Some(v as isize)
    }

    fn to_i8(&self) -> Option<i8> {
        let v = self.to_i64()?;
        if v >= -128 && v <= 127 { Some(v as i8) } else { None }
    }

    fn to_i16(&self) -> Option<i16> {
        let v = self.to_i64()?;
        if v >= -32768 && v <= 32767 { Some(v as i16) } else { None }
    }

    fn to_i32(&self) -> Option<i32> {
        let v = self.to_i64()?;
        if v >= -2147483648 && v <= 2147483647 { Some(v as i32) } else { None }
    }

    fn to_i128(&self) -> Option<i128> {
        let v = self.to_i64()?;
        Some(v as i128)
    }

    fn to_usize(&self) -> Option<usize> {
        let v = self.to_u64()?;
        Some(v as usize)
    }

    fn to_u8(&self) -> Option<u8> {
        let v = self.to_u64()?;
        if v <= 255 { Some(v as u8) } else { None }
    }

    fn to_u16(&self) -> Option<u16> {
        let v = self.to_u64()?;
        if v <= 65535 { Some(v as u16) } else { None }
    }

    fn to_u32(&self) -> Option<u32> {
        let v = self.to_u64()?;
        if v <= 4294967295 { Some(v as u32) } else { None }
    }

    fn to_u128(&self) -> Option<u128> {
        let v = self.to_u64()?;
        Some(v as u128)
    }

    fn to_f64(&self) -> Option<f64> {
        match self.to_i64() {
            Some(i) => Some(i as f64),
            None => match self.to_u64() {
                Some(u) => Some(u as f64),
                None => None,
            },
        }
    }
}

/// Float to int truncates toward zero, so a value converts when it lies
/// in the open range (MIN - 1, MAX + 1); upstream writes the bounds per
/// target width as below (2^63 and 2^127 are exact in f64). NaN fails
/// every comparison and gives None.
impl ToPrimitive for f64 {
    fn to_i64(&self) -> Option<i64> {
        let x = *self;
        if x >= -9223372036854775808.0 && x < 9223372036854775808.0 { Some(x as i64) } else { None }
    }

    fn to_u64(&self) -> Option<u64> {
        let x = *self;
        if x > -1.0 && x < 18446744073709551616.0 { Some(x as u64) } else { None }
    }

    fn to_isize(&self) -> Option<isize> {
        let x = *self;
        if x >= -9223372036854775808.0 && x < 9223372036854775808.0 { Some(x as isize) } else { None }
    }

    fn to_i8(&self) -> Option<i8> {
        let x = *self;
        if x > -129.0 && x < 128.0 { Some(x as i8) } else { None }
    }

    fn to_i16(&self) -> Option<i16> {
        let x = *self;
        if x > -32769.0 && x < 32768.0 { Some(x as i16) } else { None }
    }

    fn to_i32(&self) -> Option<i32> {
        let x = *self;
        if x > -2147483649.0 && x < 2147483648.0 { Some(x as i32) } else { None }
    }

    fn to_i128(&self) -> Option<i128> {
        let x = *self;
        if x >= -170141183460469231731687303715884105728.0 && x < 170141183460469231731687303715884105728.0 {
            Some(x as i128)
        } else {
            None
        }
    }

    fn to_usize(&self) -> Option<usize> {
        let x = *self;
        if x > -1.0 && x < 18446744073709551616.0 { Some(x as usize) } else { None }
    }

    fn to_u8(&self) -> Option<u8> {
        let x = *self;
        if x > -1.0 && x < 256.0 { Some(x as u8) } else { None }
    }

    fn to_u16(&self) -> Option<u16> {
        let x = *self;
        if x > -1.0 && x < 65536.0 { Some(x as u16) } else { None }
    }

    fn to_u32(&self) -> Option<u32> {
        let x = *self;
        if x > -1.0 && x < 4294967296.0 { Some(x as u32) } else { None }
    }

    fn to_u128(&self) -> Option<u128> {
        let x = *self;
        if x > -1.0 && x < 340282366920938463463374607431768211456.0 { Some(x as u128) } else { None }
    }

    fn to_f64(&self) -> Option<f64> {
        Some(*self)
    }
}
