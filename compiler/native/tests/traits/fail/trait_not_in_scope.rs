// rustc-error: E0599
// lust-expect: L0104
mod m {
    pub trait Hidden { fn hid(&self) -> u8; }
    impl Hidden for u8 { fn hid(&self) -> u8 { *self + 1 } }
}
fn main() { let x: u8 = 4; println!("{}", x.hid()); }
