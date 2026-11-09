// A closure that captures nothing becomes a fn pointer where one is
// expected: a binding, an argument, a field, a constructor of a
// generic type, and an adapter type named in a return type.
struct Ops {
    op: fn(i32, i32) -> i32,
}

struct Via<T>(fn(T) -> i32);

struct Pairs {
    v: Vec<(u8, char)>,
}

impl Pairs {
    fn keys(&self) -> std::iter::Map<std::slice::Iter<'_, (u8, char)>, fn(&(u8, char)) -> u8> {
        self.v.iter().map(|p| p.0)
    }
}

fn apply(f: fn(u8) -> u8, x: u8) -> u8 {
    f(x)
}

fn main() {
    let p: fn(u8) -> u8 = |x| x + 1;
    let q = apply(|x| x * 3, 5);
    let ops = [Ops { op: |a, b| a + b }, Ops { op: |a, b| a * b }];
    let r: Vec<i32> = ops.iter().map(|o| (o.op)(3, 4)).collect();
    let big: fn(u64) -> (u64, u64, u64, u64, u64) = |x| (x, x + 1, x + 2, x + 3, x + 4);
    let via: Via<u8> = Via(|x| x as i32 + 1);
    let pairs = Pairs { v: vec![(1, 'a'), (2, 'b')] };
    let ks: Vec<u8> = pairs.keys().collect();
    println!("{} {} {:?} {:?} {} {:?}", p(4), q, r, big(1), (via.0)(4), ks);
}
