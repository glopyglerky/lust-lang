mod m {
    pub trait Hidden { fn hid(&self) -> u8; }
    impl Hidden for u8 { fn hid(&self) -> u8 { *self + 1 } }
}
fn g<T: m::Hidden>(t: &T) -> u8 { t.hid() }
fn main() { println!("{}", g(&4u8)); }
