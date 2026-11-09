// A fn item held in a local, field, element, or call result coerces to a
// fn pointer: the pointer is the item's trampoline, never the zero-sized
// item value (hunt2 fn_item_local_to_fn_ptr_null).
fn dbl(x: i32) -> i32 { x * 2 }
fn inc(x: i32) -> i32 { x + 1 }
fn id<T>(t: T) -> T { t }
fn same<T: Copy>(x: T) -> T { x }
fn apply(p: fn(i32) -> i32, v: i32) -> i32 { p(v) }
struct H { f: fn(i32) -> i32 }
fn main() {
    let f = dbl;
    let g: fn(i32) -> i32 = f;
    println!("{}", g(5));
    println!("{}", apply(f, 6));
    let v: Vec<fn(i32) -> i32> = vec![f, inc];
    println!("{} {}", v[0](1), v[1](1));
    let h = H { f };
    println!("{}", (h.f)(7));
    let c = f as fn(i32) -> i32;
    println!("{}", c(8));
    let a = [f, f];
    let k: fn(i32) -> i32 = a[0];
    println!("{}", k(9));
    let t = (f, 1);
    let m: fn(i32) -> i32 = t.0;
    println!("{} {}", m(10), t.1);
    let n: fn(i32) -> i32 = id(f);
    println!("{}", n(11));
    let q = same::<i32>;
    let r: fn(i32) -> i32 = q;
    println!("{}", r(12));
    let s: fn(i32) -> i32 = { let z = inc; z };
    println!("{}", s(13));
    let o: Option<fn(i32) -> i32> = Some(f);
    println!("{}", o.unwrap()(3));
    let mut w: fn(i32) -> i32 = inc;
    println!("{}", w(4));
    w = f;
    println!("{}", w(4));
}
