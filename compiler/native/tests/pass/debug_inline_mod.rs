// Derived Debug prints source names for types in inline modules too.
mod m0 {
    #[derive(Debug)]
    pub struct Stats {
        pub n: i64,
    }

    pub mod inner {
        #[derive(Debug)]
        pub struct Deep(pub i64);
    }

    #[derive(Debug)]
    pub struct Wrap<T> {
        pub inner: T,
    }
}

fn main() {
    println!("{:?}", m0::Stats { n: 3 });
    println!("{:?}", m0::inner::Deep(4));
    println!("{:?}", m0::Wrap { inner: m0::Stats { n: 5 } });
}
