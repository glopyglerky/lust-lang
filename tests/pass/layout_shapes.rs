// Tuples, arrays, zero-sized fields, a single-variant enum, and large
// returns under layout-driven codegen.
#[derive(Debug, PartialEq)]
enum Solo {
    Only(u8, String),
}

struct Unit2;
struct HasZst {
    z: Unit2,
    n: u32,
    arr: [u16; 0],
    b: bool,
}

#[derive(Debug, PartialEq, Clone)]
struct Big {
    a: u64,
    s: String,
    t: (u8, u64, u8),
    v: Vec<u16>,
}

fn make_big() -> Big {
    let mut v: Vec<u16> = Vec::new();
    v.push(5);
    v.push(6);
    Big { a: 99, s: String::from("big"), t: (1, 2, 3), v }
}

fn main() {
    let s = Solo::Only(4, String::from("solo"));
    match s {
        Solo::Only(n, text) => println!("{} {}", n, text),
    }

    let h = HasZst { z: Unit2, n: 77, arr: [], b: true };
    println!("{} {}", h.n, h.b);
    println!("{}", h.arr.len());

    let tup: (u8, u64, u8) = (10, 20, 30);
    println!("{} {} {}", tup.0, tup.1, tup.2);
    let pair = (String::from("x"), [3u8, 1, 4]);
    println!("{} {}", pair.0, pair.1[2]);

    let big = make_big();
    let big2 = big.clone();
    println!("{} {} {:?} {} {}", big.a, big.s, big.t, big.v.len(), big == big2);
    println!("{:?}", big.v);

    // An array struct in a tuple and an enum payload of raw arrays.
    let grids: Vec<[u32; 2]> = vec![[1, 2], [3, 4]];
    println!("{} {}", grids[0][1], grids[1][0]);
}
