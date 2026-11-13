#[cfg(bs_a)]
const A: bool = true;
#[cfg(not(bs_a))]
const A: bool = false;
fn main() {
    println!("{} {}", env!("BS_VAL"), A);
}
