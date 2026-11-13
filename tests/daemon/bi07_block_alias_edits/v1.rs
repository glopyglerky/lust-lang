fn sizes() -> (usize, u64) {
    type C = u8;
    #[derive(Debug, Default)]
    struct H { c: C, v: Vec<C> }
    let h = H { c: 255, v: vec![1] };
    (std::mem::size_of::<H>(), h.c as u64 + h.v.len() as u64)
}
fn main() { println!("{:?}", sizes()); }
