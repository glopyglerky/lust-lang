// A tuple struct constructor passed as a fn value (`.map(Self)`) in a
// submodule, used and unused. `lust build` checks std's `Option::map` with
// the constructor as `F`.
mod inner {
    #[derive(Clone, Copy, Debug)]
    pub struct Rev(u64);

    impl Rev {
        pub fn first() -> Rev {
            Rev(1)
        }
        fn unused(self) -> Option<Self> {
            self.0.checked_add(1).map(Self)
        }
        pub fn next(self) -> Option<Self> {
            self.0.checked_add(1).map(Self)
        }
    }

    #[derive(Debug)]
    pub struct Open(pub u8);
}

fn main() {
    println!("{:?}", inner::Rev::first().next());
    println!("{:?}", Some(3).map(inner::Open));
    let v: Vec<inner::Open> = vec![1u8, 2].into_iter().map(inner::Open).collect();
    println!("{:?}", v);
}
