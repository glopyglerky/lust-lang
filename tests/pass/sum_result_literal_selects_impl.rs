// A sum result fixed only as a literal: rustc selects the one `Sum<Item>`
// impl the integer variable fits, so `.unwrap_or(0)` here is a `u64`.
fn main() {
    let v = vec![1u64 << 40, 2];
    let r = Some(&v).map(|v| v.iter().sum()).unwrap_or(0);
    let w = vec![1.5f32, 2.25];
    let f = Some(&w).map(|w| w.iter().copied().sum()).unwrap_or(0.0);
    let u = vec![200u8, 50];
    let p = Some(&u).map(|u| u.iter().map(|&x| x / 10).product()).unwrap_or(1);
    println!("{} {} {}", r, f, p);
    println!("{} {}", arms(&[30, 40]), arms(&[]));
}

// The first arm makes the result a literal variable before `sum` runs.
fn arms(v: &[u16]) -> String {
    let r = match v.len() {
        0 => 0,
        _ => v.iter().sum(),
    };
    let p = if v.is_empty() { 1 } else { v.iter().product() };
    format!("{} {}", r, p)
}

