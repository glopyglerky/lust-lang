// A borrowing std iterator type by path (`std::slice::Iter<'a, T>`) in a
// signature, with its lifetime.
fn first<'a>(v: &'a [u32]) -> std::slice::Iter<'a, u32> {
    v.iter()
}

fn main() {
    println!("{:?}", first(&[1, 2]).next());
}
