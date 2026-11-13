// The overflow and unconditional-panic lints run before mono, so a
// value only a substitution makes known must not reject the build.
// The bad operations sit in branches that never run.
fn add<const N: u8>(b: bool) -> u8 {
    if b { N + 200 } else { 1 }
}

fn div<const N: u8>(b: bool) -> u8 {
    if b { 10 / N } else { 2 }
}

fn idx<const N: usize>(b: bool) -> u8 {
    let a = [3u8, 4];
    if b { a[N] } else { 5 }
}

fn main() {
    println!("{} {} {}", add::<100>(false), div::<0>(false), idx::<9>(false));
}
