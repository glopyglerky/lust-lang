// The mixed-sign integer methods in const context, on the kinds std has
// them on: `_signed` on unsigned types, `_unsigned` on signed types.
const U: u8 = 250;
const I: i8 = -100;
fn main() {
    println!("{:?}", const { U.wrapping_add_signed(10) });
    println!("{:?}", const { U.checked_add_signed(-128) });
    println!("{:?}", const { U.saturating_sub_signed(-10) });
    println!("{:?}", const { I.wrapping_add_unsigned(250) });
    println!("{:?}", const { I.checked_sub_unsigned(28) });
    println!("{:?}", const { I.saturating_add_unsigned(255) });
    println!("{:?}", const { I.saturating_neg() });
}
