// `Some`, `Ok`, and `Err` as values: constructor items that iterator
// adapters, generic Fn bounds, and plain calls take. A module item
// named `Some` shadows the prelude's.
fn apply<F: Fn(u32) -> Option<u32>>(f: F, x: u32) -> Option<u32> {
    f(x)
}

fn twice<T, R, F: FnMut(T) -> R>(mut f: F, a: T, b: T) -> (R, R) {
    (f(a), f(b))
}

mod shadow {
    #[allow(non_snake_case)]
    fn Some(x: u32) -> u32 {
        x + 100
    }

    pub fn run() -> Vec<u32> {
        vec![1u32, 2].into_iter().map(Some).collect()
    }
}

fn main() {
    let v: Vec<Option<u32>> = vec![1u32, 2].into_iter().map(Some).collect();
    println!("{v:?}");
    let r: Vec<Result<u8, String>> = vec![3u8, 4].into_iter().map(Ok).collect();
    println!("{r:?}");
    let e: Vec<Result<(), u8>> = vec![5u8].into_iter().map(Err).collect();
    println!("{e:?}");
    println!("{:?}", apply(Some, 6));
    let f = Some;
    println!("{:?} {:?}", f(7u8), f(8));
    let q = Option::<i64>::Some;
    println!("{:?}", q(-9));
    let ok = Result::<u8, bool>::Ok;
    println!("{:?}", ok(10));
    println!("{:?}", twice(Err::<(), u8>, 1, 2));
    println!("{:?}", twice(Err::<(), &str>, "a", "b"));
    let pick = if v.len() > 1 { Some } else { Some };
    println!("{:?}", pick(11u16));
    println!("{:?}", shadow::run());
}
