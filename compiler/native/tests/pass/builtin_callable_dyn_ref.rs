// A `&dyn Fn` or `&mut dyn FnMut` as the function of a builtin that
// calls it (ts-rust goport.rs `guard(emit_file)` with `emit_file: &dyn
// Fn() -> EmitResult`): `catch_unwind`, `get_or_init`, through a generic
// wrapper, with a large return and a zero-sized argument.
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::OnceLock;

#[derive(Debug, Default, Clone)]
struct Big {
    a: [u64; 6],
    s: String,
}

fn guard<T: Default>(f: impl FnOnce() -> T) -> T {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(value) => value,
        Err(_) => T::default(),
    }
}

fn wrap_all(wrap: fn(&dyn Fn() -> Big) -> Big, n: u64) -> Big {
    wrap(&|| Big { a: [n; 6], s: format!("n{n}") })
}

fn init<F: FnOnce() -> String>(c: &OnceLock<String>, f: F) -> &String {
    c.get_or_init(f)
}

fn main() {
    let b = wrap_all(|f| guard(f), 3);
    println!("{:?}", b);
    let k = 7;
    let small: &dyn Fn() -> i32 = &|| k * 2;
    println!("{}", guard(small));
    let mut count = 0;
    {
        let mut bump = || {
            count += 1;
            count
        };
        let m: &mut dyn FnMut() -> i32 = &mut bump;
        println!("{}", guard(m));
    }
    println!("count {count}");
    let c = OnceLock::new();
    let name = String::from("once");
    let g: &dyn Fn() -> String = &|| name.clone();
    println!("{}", init(&c, g));
    println!("{}", init(&c, g));
    let unit: &dyn Fn(()) -> u8 = &|()| 9;
    let r = catch_unwind(AssertUnwindSafe(|| unit(())));
    println!("{:?}", r.ok());
}
