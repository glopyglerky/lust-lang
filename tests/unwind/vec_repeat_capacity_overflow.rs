// `vec![x; n]` and `resize` with a capacity overflow under panic=unwind: nothing is
// written after the panic, and the value drops as in Rust (lust segfaulted).
struct D(u8);
impl Drop for D { fn drop(&mut self) { println!("drop {}", self.0); } }
impl Clone for D { fn clone(&self) -> D { D(self.0 + 10) } }
fn main() {
    let r = std::panic::catch_unwind(|| { let v = vec![0u32; usize::MAX]; v.len() });
    println!("a {}", r.is_err());
    let r = std::panic::catch_unwind(|| { let v = vec![D(1); usize::MAX]; v.len() });
    println!("b {}", r.is_err());
    let r = std::panic::catch_unwind(|| { let mut v = vec![D(2)]; v.resize(usize::MAX, D(3)); v.len() });
    println!("c {}", r.is_err());
    let r = std::panic::catch_unwind(|| { let mut v = vec![7u64]; v.resize(usize::MAX, 1); v.len() });
    println!("d {}", r.is_err());
}
