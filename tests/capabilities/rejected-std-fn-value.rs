// capability: rejected: this std function used as a value
// expect: L9001

fn main() {
    let f = std::mem::size_of::<u32>;
    println!("{}", f());
}
