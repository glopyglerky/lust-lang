// Mixed-sign arithmetic of a signed receiver: the room below for checked/overflowing/
// saturating_sub_unsigned is x - MIN, which tf-build found unmasked on the sign-extended word
// (i8/i16/i32 missed the overflow: (-3i16).saturating_sub_unsigned(32768) gave 32765).
fn main() {
    for x in [i8::MIN, -100, -3, -1, 0, 1, 5, 100, i8::MAX] {
        for y in [0u8, 1, 3, 100, 127, 128, 129, 200, 255] {
            println!("{x} {y} {} {:?} {:?} {}", x.saturating_sub_unsigned(y), x.checked_sub_unsigned(y), x.overflowing_sub_unsigned(y), x.wrapping_sub_unsigned(y));
            println!("{} {:?} {:?}", x.saturating_add_unsigned(y), x.checked_add_unsigned(y), x.overflowing_add_unsigned(y));
        }
    }
    for x in [i16::MIN, -19616, -3, 0, 7, i16::MAX] {
        for y in [0u16, 1, 32767, 32768, 46718, 58521, 65535] {
            println!("{} {:?} {:?}", x.saturating_sub_unsigned(y), x.checked_sub_unsigned(y), x.overflowing_sub_unsigned(y));
        }
    }
    for x in [i32::MIN, -5, 0, 9, i32::MAX] {
        for y in [0u32, 1, 2147483647, 2147483648, 4294967295] {
            println!("{} {:?} {:?} {}", x.saturating_sub_unsigned(y), x.checked_sub_unsigned(y), x.overflowing_sub_unsigned(y), x.saturating_add_unsigned(y));
        }
    }
    for x in [i64::MIN, -5, 0, i64::MAX] {
        for y in [0u64, 1, 9223372036854775807, 9223372036854775808, u64::MAX] {
            println!("{} {:?} {:?}", x.saturating_sub_unsigned(y), x.checked_sub_unsigned(y), x.overflowing_sub_unsigned(y));
        }
    }
    for x in [0u8, 3, 200, 255] {
        for y in [i8::MIN, -1, 0, 1, 100, i8::MAX] {
            println!("{} {:?} {:?} {}", x.saturating_add_signed(y), x.checked_add_signed(y), x.overflowing_add_signed(y), x.wrapping_add_signed(y));
        }
    }
}
