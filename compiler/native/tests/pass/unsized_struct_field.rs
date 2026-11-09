// A struct whose only field is a slice is an unsized type (valid Rust);
// lust lays it out as its field, as for std::path::Path (W-SHIM).
struct S {
    data: [i64],
}

fn len(s: &S) -> usize {
    s.data.len()
}

fn main() {
    println!("hi");
}
