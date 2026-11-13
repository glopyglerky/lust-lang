mod a;
mod m0;

mod inl {
    #[derive(Debug)]
    pub struct Inl(pub i64);
}

#[derive(Debug)]
struct Outer<T> {
    w: m0::Wrap<T>,
}

fn main() {
    println!("{:?}", m0::Stats { n: 3 });
    println!("{:?}", m0::Pair(1, true));
    println!("{:?}", m0::Marker);
    println!("{:?}", m0::Empty {});
    println!("{:?} {:?} {:?}", m0::Shape::Dot, m0::Shape::Circle(5), m0::rect());
    println!("{:?}", a::b::Deep { v: 1 });
    println!("{:?}", a::Stats { total: 9 });
    println!("{:?}", inl::Inl(7));
    let o = Outer { w: m0::Wrap { inner: m0::Stats { n: 3 } } };
    println!("{:?}", o);
    println!("{:?}", vec![m0::Stats { n: 1 }, m0::Stats { n: 2 }]);
    println!("{:?}", Some(m0::Stats { n: 4 }));
    let r: Result<i64, m0::Stats> = Err(m0::Stats { n: 8 });
    println!("{:?}", r);
    println!("{:?}", (m0::Pair(2, false), 1));
    println!("{:?}", &m0::Stats { n: 5 });
    let s = format!("{:?}", m0::Stats { n: 6 });
    println!("{s}");
    let u: Option<m0::Stats> = Some(m0::Stats { n: 10 });
    println!("{:?}", u.unwrap());
}
