// Whole-value moves and borrowed matches of Drop types: the Drop impl
// runs once per value, at the right scope exit. Moving a String field
// out of a struct without a Drop impl stays legal.
struct Guard {
    name: String,
}

impl Drop for Guard {
    fn drop(&mut self) {
        println!("drop [{}]", self.name);
    }
}

struct Text {
    a: String,
    b: String,
}

fn take(g: Guard) {
    println!("took {}", g.name);
}

fn main() {
    let g = Guard { name: "one".to_string() };
    take(g);
    let o = Some(Guard { name: "two".to_string() });
    match &o {
        Some(g) => println!("borrowed {}", g.name),
        None => println!("none"),
    }
    let t = Text { a: "aa".to_string(), b: "bb".to_string() };
    let a = t.a;
    println!("moved {} kept {}", a, t.b);
    println!("end");
}
