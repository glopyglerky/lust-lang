// Sorting, searching, and min/max over types with derived Ord: the
// field order decides, then the variant order for enums.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
struct P(i32, u8);
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
enum Shape {
    Dot,
    Line(u32),
    Named { name: String, sides: u8 },
}
fn main() {
    let mut v = vec![P(2, 1), P(1, 9), P(2, 0), P(1, 9)];
    v.sort();
    println!("{:?} {:?}", v, v.binary_search(&P(2, 0)));
    v.sort_unstable_by(|a, b| b.cmp(a));
    println!("{:?} {:?} {:?}", v, v.iter().max(), v.iter().min());
    let mut s = vec![
        Shape::Named { name: String::from("b"), sides: 3 },
        Shape::Line(2),
        Shape::Dot,
        Shape::Named { name: String::from("a"), sides: 9 },
        Shape::Line(1),
    ];
    s.sort();
    println!("{:?}", s);
    let longest = s.iter().max_by_key(|x| match x {
        Shape::Named { sides, .. } => *sides,
        _ => 0,
    });
    println!("{:?}", longest);
    let mut pairs = vec![(P(1, 1), String::from("x")), (P(0, 5), String::from("y"))];
    pairs.sort_by_key(|p| p.0);
    println!("{:?}", pairs);
}
