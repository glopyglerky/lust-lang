// Full std module paths for items lust supports: the `core::mem`
// byte-move family, `core::cmp::max`/`min` on primitives, builtin
// type constructors and variants, and variant patterns.
struct D(i32);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

fn f<T>(a: &mut T, b: &mut T) {
    std::mem::swap(a, b);
}

fn main() {
    let a = D(1);
    std::mem::drop(a);
    println!("mid");
    drop(D(2));
    core::mem::drop(D(3));
    println!("after drops");

    let mut x = String::from("a");
    let mut y = String::from("b");
    f(&mut x, &mut y);
    println!("{} {}", x, y);
    let old = std::mem::replace(&mut x, String::from("c"));
    println!("{} {}", old, x);
    println!("{}|{}", std::mem::take(&mut x), x);

    let mut v = vec![1, 2];
    println!("{:?} {:?}", std::mem::take(&mut v), v);
    let mut o = Some(3);
    println!("{:?} {:?}", std::mem::take(&mut o), o);
    let mut n = 5;
    println!("{} {}", std::mem::take(&mut n), n);

    println!("{} {}", std::cmp::max(2, 5), std::cmp::min(2, 5));
    println!("{}", core::cmp::max(10u128, 3u128));
    println!("{}", std::cmp::max::<i32>(-1, 1));

    let w: std::vec::Vec<i32> = std::vec::Vec::new();
    println!("{}", w.len());
    let s = std::string::String::from("hi");
    println!("{}", s);
    let q: std::option::Option<i32> = std::option::Option::Some(4);
    match q {
        std::option::Option::Some(v) => println!("{}", v),
        std::option::Option::None => println!("none"),
    }
    let r: Result<i32, i32> = core::result::Result::Ok(6);
    match r {
        core::result::Result::Ok(v) => println!("{}", v),
        core::result::Result::Err(e) => println!("err {}", e),
    }
    println!("{}", std::mem::size_of::<i64>());
}
