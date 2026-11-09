fn main() {
    #[derive(Debug)]
    enum W { A }
    fn W() -> u8 { 5 }
    println!("{:?} {}", W::A, W());
}
