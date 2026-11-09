pub fn helper(v: Vec<u32>) -> (u32, &'static u8) {
    let first = match v.into_iter().next() {
        Some(x) => x,
        None => 0,
    };
    let r: &'static u8 = if first > 100 { &0 } else { &42 };
    (first, r)
}
