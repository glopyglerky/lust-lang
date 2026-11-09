// A struct field whose value unsizes `&[T; N]` (or `&Vec<T>`) to `&[T]`:
// the field temporary has the slice type (it had the array reference
// type, which tcc rejected; W-STD-B's repro).
struct W<'a> {
    s: &'a [&'a str],
}
struct V<'a> {
    items: &'a [u8],
}
fn main() {
    let src = ["a", "b"];
    let w = W { s: &src };
    let v = vec![1u8, 2, 3];
    let x = V { items: &v };
    let y = V { items: &[4, 5] };
    println!("{} {} {} {}", w.s.len(), w.s[1], x.items.len(), y.items[1]);
}
