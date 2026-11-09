// Core `From`/`Into` between scalar types: exactly rustc's
// lossless set, in method form, path form, and through bounds.
fn via_into<T: Into<u32>>(x: T) -> u32 {
    x.into()
}

fn via_from<T: From<u8>>(x: u8) -> T {
    T::from(x)
}

fn main() {
    let a: u32 = 5u8.into();
    let b: u128 = 5u64.into();
    let c: f64 = 3u32.into();
    let d: f32 = 2i16.into();
    let e: u32 = 'a'.into();
    let f: char = 65u8.into();
    let g: f64 = 1.5f32.into();
    let h: i64 = true.into();
    let i: usize = 7u16.into();
    let j: isize = 2i8.into();
    println!("{} {} {} {} {} {} {} {} {} {}", a, b, c, d, e, f, g, h, i, j);
    println!("{} {}", u32::from(5u8), i64::from(2i8));
    println!("{} {}", via_into(5u8), via_from::<u32>(5u8));
}
