#[repr(u8)]
enum Small {
    A = 3,
    B,
    C = 10,
}

enum Signed {
    Neg = -5,
    Zero = 0,
    Big = 1000,
    Next,
}

#[repr(C)]
struct Point {
    x: u8,
    y: u64,
}

#[repr(packed)]
struct Packed {
    a: u8,
    b: u64,
}

#[repr(transparent)]
struct Wrap(u64);

#[repr(C, align(16))]
struct Aligned {
    v: u8,
}

fn main() {
    println!("{} {} {}", Small::A as i32, Small::B as i32, Small::C as i32);
    println!("{} {} {}", Signed::Neg as i64, Signed::Big as i64, Signed::Next as i64);
    match Signed::Neg {
        Signed::Neg => println!("neg"),
        _ => println!("other"),
    }
    let p = Point { x: 1, y: 2 };
    println!("{} {}", p.x, p.y);
    let pk = Packed { a: 3, b: 4 };
    let (pa, pb) = (pk.a, pk.b);
    println!("{} {}", pa, pb);
    let w = Wrap(9);
    println!("{}", w.0);
    let al = Aligned { v: 5 };
    println!("{}", al.v);
}
