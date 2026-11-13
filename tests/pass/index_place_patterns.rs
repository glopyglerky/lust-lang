// Matching on an index place without moving out of it: `ref`
// bindings, literal tests, Copy elements, and a Copy field read.
// The array keeps its elements and drops them whole.
struct D(u8);

impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

struct S {
    f: D,
    g: u8,
}

fn main() {
    let a = [D(1), D(2)];
    match a[1] {
        ref x => println!("ref {}", x.0),
    }
    match a[1] {
        D(5) => println!("five"),
        _ => println!("other"),
    }
    println!("mid");
    let mut b = [D(3), D(4)];
    match b[0] {
        ref mut x => x.0 = 9,
    }
    println!("mut {}", b[0].0);
    let c = [10u8, 20];
    match c[1] {
        x => println!("copy {}", x),
    }
    let s = [S { f: D(7), g: 8 }];
    let g = s[0].g;
    println!("g {}", g);
    println!("end");
}
