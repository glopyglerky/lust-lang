// Inline const blocks in generic bodies (ARCH 2.13): the template
// checks the block in generic form, each instance evaluates it. A
// `const { .. }` statement needs no `;`, like other block-like ones.
fn check<V>(v: Vec<V>) -> usize {
    const {
        assert!(std::mem::size_of::<Option<V>>() <= 32, "too big")
    }
    v.len()
}

fn size<T>() -> usize {
    const { std::mem::size_of::<T>() }
}

fn main() {
    const { assert!(std::mem::size_of::<u8>() == 1) }
    let x = const { 3 + 4 };
    println!("{} {} {} {}", check(vec![1u8, 2]), size::<u16>(), size::<(u8, u32)>(), x);
}
