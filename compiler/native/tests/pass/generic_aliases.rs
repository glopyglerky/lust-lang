// Generic type aliases: args substitute into the target, defaults
// fill omitted trailing args, and an alias may name a generic type
// applied to arguments.
struct W<T>(T);

type Twice<T> = (T, T);
type Res<T> = Result<T, String>;
type WInt = W<i32>;
type Bytes<T = u8> = Vec<T>;
type Nested<T> = Twice<Option<T>>;
type Bounded<T: Clone> = Vec<T>;

fn check(x: i32) -> Res<i32> {
    if x > 0 { Ok(x) } else { Err("neg".to_string()) }
}

fn main() {
    let t: Twice<u8> = (1, 2);
    let w: WInt = W(3);
    let b: Bytes = vec![255];
    let c: Bytes<u16> = vec![256];
    let n: Nested<char> = (Some('a'), None);
    let s: Bounded<String> = Vec::new();
    println!("{} {} {:?} {:?}", t.0 + t.1, w.0, check(1), check(-1));
    println!("{} {} {:?} {}", b[0].checked_add(1).is_none(), c[0], n, s.len());
}
