// `#[non_exhaustive]` on a struct, an enum, and a variant. Inside
// the defining crate it changes nothing: literals, constructors,
// patterns without `..`, and matches without a wildcard all work,
// from the same module and from another module of the crate.
mod inner {
    #[non_exhaustive]
    pub struct Opts {
        pub level: i64,
    }
    #[non_exhaustive]
    pub struct Pair(pub i64);
    #[non_exhaustive]
    pub struct Unit;
    #[non_exhaustive]
    pub enum Kind {
        A,
        B(i64),
    }
    pub enum Shape {
        Dot,
        #[non_exhaustive]
        Rect { w: i64, h: i64 },
        #[non_exhaustive]
        Wrap(i64),
    }
    pub fn area(s: &Shape) -> i64 {
        match s {
            Shape::Dot => 0,
            Shape::Rect { w, h } => w * h,
            Shape::Wrap(n) => *n,
        }
    }
    pub fn kind_name(k: &Kind) -> i64 {
        match k {
            Kind::A => 1,
            Kind::B(n) => *n,
        }
    }
}

fn main() {
    let o = inner::Opts { level: 3 };
    let inner::Opts { level } = &o;
    let p = inner::Pair(4);
    let _u = inner::Unit;
    let k = inner::Kind::B(5);
    let s = inner::Shape::Rect { w: 2, h: 6 };
    let t = inner::Shape::Wrap(7);
    println!(
        "{} {} {} {} {}",
        level,
        p.0,
        inner::kind_name(&k),
        inner::area(&s),
        inner::area(&t)
    );
}
