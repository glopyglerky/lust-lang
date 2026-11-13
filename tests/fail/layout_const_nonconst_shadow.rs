// expect: L0110
mod core {
    pub mod mem {
        pub fn size_of<T>() -> usize { 4 }
    }
}
fn bytes() -> [u8; core::mem::size_of::<u32>()] { [0; 4] }
fn main() {}
