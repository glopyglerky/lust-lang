pub fn get(v: Vec<u32>) -> (u32, &'static [u8; 3], &'static u8) {
    let (n, r) = crate::b::helper(v);
    (n, &[1, 2, 3], r)
}
