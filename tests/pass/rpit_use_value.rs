// Iterator provided methods (`sum`) on an `impl Iterator` value, as on a type parameter.
fn evens() -> impl Iterator<Item = u32> {
    vec![2u32, 4].into_iter()
}

fn main() {
    let s: u32 = evens().sum();
    println!("{}", s);
}
