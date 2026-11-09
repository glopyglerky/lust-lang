#[cfg(bs)]
const X: u32 = 1;
#[cfg(not(bs))]
const X: u32 = 2;
fn main() {
    println!("{}", X);
}
