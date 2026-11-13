// Partial moves in generic instance bodies, at a type with a user
// `Drop` impl.
struct D(u8);

impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

struct Pair<T> {
    a: T,
    b: T,
}

fn second<T>(p: Pair<T>) -> T {
    let Pair { a: _, b } = p;
    println!("second");
    b
}

fn pick<T>(o: Option<T>, alt: T) -> T {
    match o {
        Some(x) => x,
        None => alt,
    }
}

fn main() {
    let b = second(Pair { a: D(1), b: D(2) });
    println!("b {}", b.0);
    let x = pick(Some(D(3)), D(4));
    println!("x {}", x.0);
    let y = pick(None, D(5));
    println!("y {}", y.0);
}
