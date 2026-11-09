fn sizes() -> (usize, u64) {
    type C = D;
    type D = (u8, u8);
    #[derive(Debug, Default)]
    struct H { c: C, v: Vec<C> }
    let h = H { c: (1, 2), v: vec![] };
    (std::mem::size_of::<H>(), h.c.0 as u64 + h.v.len() as u64)
}
fn main() { println!("{:?}", sizes()); }
