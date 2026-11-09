struct Pair(i64, bool);

#[derive(Debug, PartialEq)]
struct Wrap((i64, i64));

fn swap(p: (i64, i64)) -> (i64, i64) {
    (p.1, p.0)
}

fn main() {
    let t = (1, "two".to_string(), true);
    println!("{} {} {}", t.0, t.1, t.2);
    let (a, _b, c) = t;
    println!("{} {}", a, c);

    let nested = ((1, 2), (3, 4));
    println!("{}", nested.0.1 + nested.1.0);
    println!("{:?}", nested);

    let s = swap((5, 9));
    println!("{} {}", s.0, s.1);
    println!("{}", s == (9, 5));

    let one: (i64,) = (42,);
    println!("{:?}", one);

    let p = Pair(7, true);
    println!("{} {}", p.0, p.1);
    let Pair(n, flag) = p;
    println!("{} {}", n, flag);

    let w = Wrap((3, 4));
    println!("{:?} {}", w, w.0.0 + w.0.1);

    match (1, 2) {
        (1, x) => println!("one {}", x),
        _ => println!("other"),
    }
    match (1, 2, 3) {
        (first, ..) => println!("{}", first),
    }
    match (1, 2, 3) {
        (.., last) => println!("{}", last),
    }

    let mut arr = [10, 20, 30];
    arr[1] = 99;
    println!("{} {} {}", arr[0], arr[1], arr[2]);
    println!("{:?}", arr);
    println!("{}", arr == [10, 99, 30]);

    let rep = [7; 4];
    println!("{:?}", rep);

    let typed: [i64; 2] = [1 + 1, 3];
    println!("{}", typed[0] + typed[1]);

    let strings = ["a".to_string(), "b".to_string()];
    println!("{} {}", strings[0], strings[1]);

    let mut total = 0;
    let mut i = 0usize;
    while i < 3 {
        total += arr[i];
        i += 1;
    }
    println!("{}", total);

    let grid = [(1, 2), (3, 4)];
    println!("{}", grid[1].1);

    let mut mp = (1, 2);
    mp.0 += 10;
    println!("{} {}", mp.0, mp.1);
}
