// derive(PartialOrd, Ord) through nesting: enums with data, Vec, Option,
// Box, tuples, and references; cmp results compared pairwise.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
struct In { x: i64, s: String }
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
enum Mid { One(In), Two { v: Vec<In>, o: Option<Box<In>> }, Three }
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
struct Out(Mid, u8, (char, bool));
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Pair(u8, u8);

fn nested() {
    let i1 = In { x: 1, s: "a".to_string() };
    let i2 = In { x: 1, s: "b".to_string() };
    let ms = vec![
        Mid::One(i1.clone()), Mid::One(i2.clone()), Mid::Three,
        Mid::Two { v: vec![i1.clone()], o: None }, Mid::Two { v: vec![i1.clone(), i2.clone()], o: None },
        Mid::Two { v: vec![i1.clone()], o: Some(Box::new(i2.clone())) },
    ];
    for a in &ms { for b in &ms { print!("{:?} ", a.cmp(b)); } println!(); }
    let o1 = Out(Mid::Three, 1, ('a', true));
    let o2 = Out(Mid::Three, 1, ('a', false));
    println!("{:?} {} {}", o1.cmp(&o2), o1 > o2, &o1 == &o1);
    println!("{}", vec![i1.clone()] < vec![i1.clone(), i2.clone()]);
    println!("{}", (i1.clone(), 2) < (i1.clone(), 3));
    println!("{:?}", Some(&i1).cmp(&Some(&i2)));
}

fn refs() {
    let a = Pair(1, 2);
    let b = Pair(1, 3);
    let (ra, rb) = (&a, &b);
    println!("{} {} {:?} {:?}", ra < rb, &&a >= &&b, ra.cmp(rb), (&ra).partial_cmp(&rb));
    let v = vec![Pair(3, 0), Pair(1, 9)];
    println!("{:?}", v[0].cmp(&v[1]));
}

fn main() {
    nested();
    refs();
}
