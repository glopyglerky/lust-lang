// A field with a Drop impl moved out of a struct drops once, with its
// new owner; the struct's other fields still drop at scope exit.
struct Guard {
    name: String,
}

impl Drop for Guard {
    fn drop(&mut self) {
        println!("drop [{}]", self.name);
    }
}

struct Pair {
    g: Guard,
    n: i32,
}

fn main() {
    let p = Pair { g: Guard { name: "g".to_string() }, n: 1 };
    let g = p.g;
    println!("got {}", g.name);
}
