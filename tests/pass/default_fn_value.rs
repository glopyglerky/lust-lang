// `Default::default` and std constructors as fn values, also as a
// static `LazyLock`'s init fn.
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

static M: LazyLock<Mutex<HashMap<i32, String>>> = LazyLock::new(Default::default);
static N: LazyLock<HashMap<i32, i32>> = LazyLock::new(HashMap::new);
static O: LazyLock<Vec<u8>> = LazyLock::new(make);
static P: LazyLock<String> = LazyLock::new(String::new,);

fn make() -> Vec<u8> {
    println!("make runs");
    vec![1, 2]
}

#[derive(Default, Debug)]
struct Q {
    a: i32,
    b: String,
}

fn mk<T: Default>() -> T {
    let f: fn() -> T = Default::default;
    f()
}

fn main() {
    M.lock().unwrap().insert(1, "x".into());
    println!("{:?} {} {}", M.lock().unwrap().get(&1), N.len(), P.len());
    println!("before O");
    println!("{:?} {:?}", *O, *O);
    let o: Option<Q> = None;
    println!("{:?}", o.unwrap_or_else(Default::default));
    let q: Q = mk();
    let v: Vec<i32> = mk();
    println!("{:?} {:?}", q, v);
    let mut m: HashMap<i32, Vec<i32>> = HashMap::new();
    m.entry(1).or_insert_with(Vec::new).push(2);
    m.entry(1).or_insert_with(Default::default).push(3);
    println!("{:?}", m);
    let f = <i32 as Default>::default;
    let g = <String as From<&str>>::from;
    println!("{} {}", f(), g("y"));
}
