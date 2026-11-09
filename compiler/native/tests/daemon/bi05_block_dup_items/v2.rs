fn main() {
    #[derive(Debug)]
    struct W(u8);
    #[derive(Debug)]
    struct W(u16);
    println!("{:?}", W(1));
}
