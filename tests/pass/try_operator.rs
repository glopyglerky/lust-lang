// The ? operator on Option (was a reject before wave A2).
fn get() -> Option<i64> {
    let x = Some(3)?;
    Some(x + 1)
}

fn main() {
    println!("{}", get().unwrap());
}
