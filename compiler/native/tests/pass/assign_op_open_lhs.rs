// `x op= y` whose left type a later use fixes waits for body end, as rustc's
// pending operator obligation does; `String += if ..` takes a `&str`.
fn main() {
    let mut pool = Vec::new();
    for &(x, bits) in &[(1u32, 1u8), (1, 2), (2, 4)] {
        match pool.last_mut() {
            Some((last, last_bits)) if *last == x => *last_bits |= bits,
            _ => pool.push((x, bits)),
        }
    }
    let keep: Vec<(u32, u8)> = pool;
    let c = keep.len() > 5;
    let mut s = String::from("a");
    s += if c { "x" } else { "y" };
    s += &String::from("q");
    println!("{:?} {s}", keep);
}
