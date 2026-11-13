// The std platform crate joins for the ExitCode return type in main's
// header. v2 edits only main's header and tail, so the item patch runs;
// with no other platform name left, a cold resolve drops the platform
// crate and the std splice, and so must the warm one (edit fuzz small
// seed 42). v3 edits a call in main's body; v4 brings the platform crate back.
#[track_caller]
fn f() -> u32 { std::panic::Location::caller().line() }
fn h(nums: &Vec<i64>, k: usize) {
    println!("{}", f());
    let o: Option<u8> = None;
    if pick(nums, k) == 30 { o.expect("boom"); }
}
#[derive(Debug, Clone, PartialEq)]
struct Point {
    x: i64,
    y: i64,
}

#[derive(Debug)]
enum Shape {
    Dot,
    Pair(i64, i64),
}

const fn seed_val() -> i64 {
    3
}

const SEED: i64 = seed_val();

static TABLE: [i64; 2] = [SEED, SEED + 1];

impl Point {
    fn sum(&self) -> i64 {
        self.x + self.y
    }

    fn bump(&mut self, d: i64) {
        self.x += d;
        self.y += d * 2;
    }
}

fn classify(s: Shape) -> i64 {
    match s {
        Shape::Dot => 0,
        Shape::Pair(a, b) => a * 10 + b,
    }
}

fn total(pts: &Vec<Point>) -> i64 {
    let mut acc = 0;
    let mut i = 0;
    while i < pts.len() {
        acc += pts[i].sum();
        i += 1;
    }
    acc
}

fn label(n: i64) -> String {
    let mut s = String::from("n=");
    s.push_str(&n.to_string());
    s
}

fn pick(v: &Vec<i64>, i: usize) -> i64 {
    v[i]
}

fn hit(n: i64) -> i64 {
    match n {
        SEED => 1,
        _ => 0,
    }
}

fn main() {
    let mut pts: Vec<Point> = Vec::new();
    pts.push(Point { x: 1, y: 2 });
    pts.push(Point { x: 3, y: 4 });
    pts[0].bump(5);
    let t = total(&pts);
    let c = classify(Shape::Pair(2, 3));
    let nums = vec![10, 20, 30];
    println!("{} {} {} {} {} {} {:?} {}", label(t), c, pick(&nums, 1), pts.len(),
             TABLE[1], hit(3), pts[0].clone(), std::mem::size_of::<Point>());
    h(&nums, 0);
}
