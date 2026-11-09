const fn triple(x: i32) -> i32 {
    x * 3
}

struct Pt {
    x: i32,
}

impl Pt {
    const fn get(&self) -> i32 {
        self.x
    }
    const fn new(x: i32) -> Pt {
        Pt { x }
    }
}

const fn sum_while(a: [i32; 3]) -> i32 {
    let mut s = 0;
    let mut i = 0;
    while i < 3 {
        s += a[i];
        i += 1;
    }
    s
}

const fn pick(x: Option<i32>) -> i32 {
    match x {
        Some(v) => v + 1,
        None => 0,
    }
}

const fn negate(x: &mut i32) {
    *x = -*x;
}

const fn len_of(s: &str) -> usize {
    s.len()
}

const fn id_string(s: String) -> String {
    s
}

const fn empty_vec() -> Vec<i32> {
    Vec::new()
}

const fn checked(x: i32) -> i32 {
    x.checked_add(1).unwrap()
}

const MAX: i32 = triple(2);

fn main() {
    println!("{}", triple(4));
    println!("{}", Pt::new(9).get());
    println!("{}", sum_while([1, 2, 3]));
    println!("{}", pick(Some(41)));
    println!("{}", pick(None));
    let mut v = 5;
    negate(&mut v);
    println!("{}", v);
    println!("{}", len_of("four"));
    println!("{}", id_string(String::from("hi")));
    println!("{}", empty_vec().len());
    println!("{}", checked(1));
    println!("{}", MAX);
}
