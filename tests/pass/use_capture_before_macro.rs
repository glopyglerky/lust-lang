// A precise-capturing `use<..>` bound in a signature is no `use` item: the
// macro and the `use` item after it in the same module stay visible
// (ts-rust ast/file_version.rs `take_data` before the `go_file_ref` macro).
mod a {
    pub struct V(pub Vec<u8>);
    impl V {
        pub fn take(&mut self) -> impl Send + use<> {
            std::mem::take(&mut self.0)
        }
    }
    pub fn first<'a, T: Copy>(x: &'a [T]) -> impl Iterator<Item = T> + use<'a, T> {
        x.iter().copied()
    }
    pub fn adder(k: u32) -> impl Fn(u32) -> u32 + use<> {
        move |x| x + k
    }
    macro_rules! twice {
        ($e:expr) => {
            $e * 2
        };
    }
    pub(crate) use twice;
    use std::collections::BTreeMap;
    pub fn table() -> i32 {
        let mut m = BTreeMap::new();
        m.insert(1, twice!(21));
        m[&1]
    }
}
fn main() {
    let mut v = a::V(vec![1, 2]);
    let _t = v.take();
    let s: u32 = a::first(&[1u32, 2, 3]).sum();
    println!("{} {} {} {} {}", v.0.len(), s, a::adder(3)(4), a::twice!(21), a::table());
}
