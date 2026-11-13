// A `use<..>` bound before a macro in the same module, on one warm daemon:
// body edits after it, then the macro moved above it, then a bad edit.
mod a {
    macro_rules! twice {
        ($e:expr) => {
            $e * 2
        };
    }
    pub fn take(v: &mut Vec<u8>) -> impl Send + use<> {
        std::mem::take(v)
    }
    pub fn calc() -> i32 {
        twice!(10) + 1
    }
}
fn main() {
    let mut v = vec![1u8, 2];
    let _t = a::take(&mut v);
    println!("{} {}", v.len(), a::calc());
}
