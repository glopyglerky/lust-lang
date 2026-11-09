pub fn f() -> u32 {
    let t = b"uu";
    let a = textual_bytes!();
    let r = textual_raw!();
    a[0] as u32 + t[1] as u32 + textual_byte!() as u32 + r[0] as u32
}
