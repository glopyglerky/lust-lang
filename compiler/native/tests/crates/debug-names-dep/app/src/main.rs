fn main() {
    println!("{:?}", base::Rooted { n: 1 });
    println!("{:?} {:?}", base::inner::Deep(2), base::inner::Kind::B(3));
    println!("{:?}", base::inner::Kind::A);
}
