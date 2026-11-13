// A destructuring `let` or a `match` binds out of a place, as in
// rustc: the parts a pattern does not move stay in the place and drop
// when its owner goes out of scope, not at the end of the inner block.
struct D(u8);

impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

struct P {
    a: D,
    b: D,
}

enum E {
    Two(D, D),
    One(D),
}

fn lets() {
    let p = P { a: D(1), b: D(2) };
    {
        let P { a, b: _ } = p;
        println!("a {}", a.0);
    }
    let t = (D(3), D(4));
    {
        let (_, y) = t;
        println!("y {}", y.0);
    }
    println!("lets end");
}

fn matches() {
    let e = E::Two(D(5), D(6));
    {
        match e {
            E::Two(a, _) => println!("got {}", a.0),
            E::One(_) => {}
        }
        println!("inner end");
    }
    let o = Some((D(7), D(8)));
    {
        if let Some((x, _)) = o {
            println!("x {}", x.0);
        }
        println!("inner end 2");
    }
    println!("matches end");
}

fn arrays(c: bool) {
    let a = [D(9), D(10)];
    if c {
        let [_, y] = a;
        println!("y {}", y.0);
    }
    println!("arrays end {}", c);
}

fn main() {
    lets();
    matches();
    arrays(true);
    arrays(false);
}
