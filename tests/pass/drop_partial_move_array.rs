// Moves of array elements by pattern and by-value iteration, at an
// element type with a user `Drop` impl and at String and Vec: the
// moved elements drop with their new owners, the rest drop with the
// array in rustc's order for a partially moved array (runs of
// unmoved elements and moved paths in reverse index order).
struct D(u8);

impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

struct S {
    arr: [D; 2],
    x: D,
}

fn pattern() {
    let a = [D(1), D(2), D(3)];
    let [x, _, z] = a;
    println!("x {} z {}", x.0, z.0);
}

fn iterate() {
    let a = [D(4), D(5), D(6)];
    for d in a {
        println!("it {}", d.0);
    }
    println!("iterate end");
}

fn in_struct() {
    let s = S { arr: [D(7), D(8)], x: D(9) };
    let [p, _] = s.arr;
    println!("p {}", p.0);
}

fn tuples() {
    let a = [(D(10), D(11)), (D(12), D(13))];
    let [(x, _), _] = a;
    println!("x {}", x.0);
}

fn owned() {
    let a = [String::from("a"), String::from("b"), String::from("c")];
    let [x, _, z] = a;
    println!("{} {}", x, z);
    let b = [vec![1u8], vec![2u8, 3]];
    for v in b {
        println!("{}", v.len());
    }
}

fn main() {
    pattern();
    iterate();
    in_struct();
    tuples();
    owned();
}
